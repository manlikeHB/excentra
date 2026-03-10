use rust_decimal::Decimal;
use std::sync::Arc;
use tokio::sync::Mutex;

use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    api::types::order::{
        OrderRequestValidationError, PlaceOrderRequest, PlaceOrderResponse, TradeInfo,
    },
    db::models::trading_pairs::DBTradingPair,
    engine::{
        exchange::Exchange,
        models::{order::Order},
    },
    error::AppError,
};

use crate::db::{
    models::order::{DBOrderSide, DBOrderType},
    queries::{self as db_queries},
};

pub struct OrderService {
    pool: PgPool,
    exchange: Arc<Mutex<Exchange>>,
}

impl OrderService {
    pub fn new(pool: PgPool, exchange: Arc<Mutex<Exchange>>) -> Self {
        OrderService { pool, exchange }
    }

    pub async fn place_order(
        &self,
        user_id: Uuid,
        body: PlaceOrderRequest,
    ) -> Result<PlaceOrderResponse, AppError> {
        let mut final_trades: Vec<TradeInfo> = Vec::new();

        // get trading pair
        let trading_pair = self.get_trading_pair(&body.symbol).await?;

        // validate balance
        let (asset, amount) = self
            .validate_and_hold_balance(&body, user_id, &trading_pair)
            .await?;

        // create order
        let mut order = Order::new(
            Uuid::new_v4(),
            user_id,
            trading_pair.id,
            body.side.into(),
            body.order_type.into(),
            body.price,
            body.quantity,
            body.quantity,
        );

        // place order
        let place_order_result;
        {
            place_order_result = self
                .exchange
                .lock()
                .await
                .place_order(trading_pair.id, &mut order);
        }

        let match_result = match place_order_result {
            Ok(res) => res,
            Err(e) => {
                // release held balance since order matching failed
                db_queries::release(&self.pool, user_id, asset, amount).await?;

                return Err(e.into());
            }
        };

        // persist order in DB
        db_queries::create_order(&self.pool, order.into()).await?;

        // persist trade in DB
        for trade in match_result.trades() {
            db_queries::create_trade(&self.pool, (*trade).into()).await?;

            final_trades.push(TradeInfo {
                price: trade.price(),
                quantity: trade.quantity(),
            });

            db_queries::transfer_on_fill(
                &self.pool,
                trade.buyer_id(),
                trade.seller_id(),
                &trading_pair.base_asset,
                &trading_pair.quote_asset,
                trade.quantity(),
                trade.price(),
            )
            .await?;
        }

        let place_order_response = PlaceOrderResponse {
            order_id: order.id(),
            status: order.status().into(),
            filled_quantity: order.quantity() - order.remaining_quantity(),
            remaining_quantity: order.remaining_quantity(),
            trades: final_trades,
        };

        Ok(place_order_response)
    }

    async fn get_trading_pair(&self, symbol: &str) -> Result<DBTradingPair, AppError> {
        match db_queries::find_by_symbol(&self.pool, symbol).await? {
            Some(pair) => Ok(pair),
            None => {
                return Err(AppError::Unprocessable(
                    "Invalid or unsupported trading pair".to_string(),
                ));
            }
        }
    }

    async fn validate_and_hold_balance<'a>(
        &self,
        body: &PlaceOrderRequest,
        user_id: Uuid,
        trading_pair: &'a DBTradingPair,
    ) -> Result<(&'a str, Decimal), AppError> {
        let (asset, amount) = match body.order_type {
            DBOrderType::Limit => {
                match body.side {
                    // check quote balance when buying base asset
                    DBOrderSide::Buy => {
                        match db_queries::get_balance(
                            &self.pool,
                            user_id,
                            &trading_pair.quote_asset,
                        )
                        .await?
                        {
                            Some(bal) => {
                                let price = match body.price {
                                    Some(p) => p,
                                    None => {
                                        return Err(
                                            OrderRequestValidationError::InvalidLimitOrder.into()
                                        );
                                    }
                                };
                                let cost = price * body.quantity;
                                if bal.available < cost {
                                    return Err(AppError::Unprocessable(format!(
                                        "Insufficient available {} balance",
                                        bal.asset
                                    )));
                                } else {
                                    // hold the quote asset
                                    db_queries::hold(
                                        &self.pool,
                                        user_id,
                                        &trading_pair.quote_asset,
                                        cost,
                                    )
                                    .await?;
                                    (&trading_pair.quote_asset, cost)
                                }
                            }
                            None => {
                                return Err(AppError::Unprocessable(format!(
                                    "Insufficient available {} balance",
                                    trading_pair.quote_asset
                                )));
                            }
                        }
                    }
                    DBOrderSide::Sell => {
                        // check base balance when selling for some quote asset
                        self.check_balance_and_hold_base(user_id, &trading_pair, body.quantity)
                            .await?
                    }
                }
            }
            DBOrderType::Market => {
                match body.side {
                    //TODO: check quote balance against what trader is willing to spend
                    DBOrderSide::Buy => {
                        return Err(AppError::BadRequest(
                            "Market buy not yet supported".to_string(),
                        ));
                    }
                    DBOrderSide::Sell => {
                        // check base balance when selling for some quote asset
                        self.check_balance_and_hold_base(user_id, &trading_pair, body.quantity)
                            .await?
                    }
                }
            }
        };

        Ok((asset, amount))
    }

    async fn check_balance_and_hold_base<'a>(
        &self,
        user_id: Uuid,
        trading_pair: &'a DBTradingPair,
        quantity: Decimal,
    ) -> Result<(&'a String, Decimal), AppError> {
        match db_queries::get_balance(&self.pool, user_id, &trading_pair.base_asset).await? {
            Some(bal) => {
                if bal.available < quantity {
                    return Err(AppError::Unprocessable(format!(
                        "Insufficient available {} balance",
                        bal.asset
                    )));
                } else {
                    // hold base asset
                    db_queries::hold(&self.pool, user_id, &trading_pair.base_asset, quantity)
                        .await?;
                    Ok((&trading_pair.base_asset, quantity))
                }
            }
            None => {
                return Err(AppError::Unprocessable(format!(
                    "Insufficient available {} balance",
                    trading_pair.base_asset
                )));
            }
        }
    }
}
