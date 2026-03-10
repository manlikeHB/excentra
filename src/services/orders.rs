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
        matcher::MatchResult,
        models::order::{Order, OrderSide},
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
        let place_order_result = self
            .exchange
            .lock()
            .await
            .place_order(trading_pair.id, &mut order);

        let match_result = match place_order_result {
            Ok(res) => res,
            Err(e) => {
                // release held balance since order matching failed
                db_queries::release(&self.pool, user_id, &asset, amount).await?;

                return Err(e.into());
            }
        };

        // persist order in DB
        let place_order_response = self
            .persist_order_and_trades(
                order,
                match_result,
                &trading_pair.base_asset,
                &trading_pair.quote_asset,
            )
            .await?;

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

    async fn validate_and_hold_balance(
        &self,
        body: &PlaceOrderRequest,
        user_id: Uuid,
        trading_pair: &DBTradingPair,
    ) -> Result<(String, Decimal), AppError> {
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
                                    (trading_pair.quote_asset.clone(), cost)
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

        Ok((asset.clone(), amount))
    }

    async fn check_balance_and_hold_base(
        &self,
        user_id: Uuid,
        trading_pair: &DBTradingPair,
        quantity: Decimal,
    ) -> Result<(String, Decimal), AppError> {
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
                    Ok((trading_pair.base_asset.clone(), quantity))
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

    async fn persist_order_and_trades(
        &self,
        order: Order,
        match_result: MatchResult,
        base_asset: &str,
        quote_asset: &str,
    ) -> Result<PlaceOrderResponse, AppError> {
        let mut tx = self.pool.begin().await?;
        let mut final_trades = Vec::new();

        db_queries::create_order(&mut *tx, order.into()).await?;

        for trade in match_result.trades() {
            // persist trade in DB
            db_queries::create_trade(&mut *tx, (*trade).into()).await?;

            // get resting order ID
            let resting_order_id = if order.id() == trade.buy_order_id() {
                trade.sell_order_id()
            } else {
                trade.buy_order_id()
            };

            // get resting Order
            let mut resting_order: Order =
                match db_queries::get_order_by_id(&self.pool, resting_order_id).await? {
                    Some(o) => o.into(),
                    None => {
                        return Err(AppError::InternalError(
                            "Invalid order ID in trade".to_string(),
                        ));
                    }
                };

            // update order state
            resting_order.reduce_quantity(trade.quantity());

            // persist resting order in DB
            db_queries::update_order_after_trade(
                &mut tx,
                resting_order.id(),
                resting_order.status().into(),
                resting_order.remaining_quantity(),
            )
            .await?;

            final_trades.push(TradeInfo {
                price: trade.price(),
                quantity: trade.quantity(),
            });

            db_queries::transfer_on_fill(
                &mut tx,
                trade.buyer_id(),
                trade.seller_id(),
                base_asset,
                quote_asset,
                trade.quantity(),
                trade.price(),
            )
            .await?;
        }

        tx.commit().await?;

        let place_order_response = PlaceOrderResponse {
            order_id: order.id(),
            status: order.status().into(),
            filled_quantity: order.quantity() - order.remaining_quantity(),
            remaining_quantity: order.remaining_quantity(),
            trades: final_trades,
        };

        Ok(place_order_response)
    }
}
