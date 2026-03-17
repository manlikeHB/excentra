use rust_decimal::Decimal;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};

use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    api::types::order::{
        OrderRequestValidationError, OrderResponse, PlaceOrderRequest, PlaceOrderResponse,
        TradeInfo,
    },
    db::models::{
        order::{DBOrder, DBOrderStatus},
        trading_pairs::DBTradingPair,
    },
    engine::{exchange::Exchange, matcher::MatchResult, models::order::Order},
    error::AppError,
    types::asset_symbol::AssetSymbol,
    ws::messages::WsEvent,
};

use crate::db::{
    models::order::{DBOrderSide, DBOrderType},
    queries::{self as db_queries},
};

pub struct OrderService {
    pool: PgPool,
    exchange: Arc<Mutex<Exchange>>,
    ws_sender: broadcast::Sender<WsEvent>,
}

impl OrderService {
    pub fn new(
        pool: PgPool,
        exchange: Arc<Mutex<Exchange>>,
        ws_sender: broadcast::Sender<WsEvent>,
    ) -> Self {
        OrderService {
            pool,
            exchange,
            ws_sender,
        }
    }

    pub async fn place_order(
        &self,
        user_id: Uuid,
        body: PlaceOrderRequest,
    ) -> Result<PlaceOrderResponse, AppError> {
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
        let symbol = AssetSymbol::new(&format!("{}/{}", base_asset, quote_asset))
            .map_err(|_| AppError::InternalError("Invalid trading pair symbol".to_string()))?;

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
                match db_queries::get_order_by_id(&mut *tx, resting_order_id).await? {
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

            // broadcast trade
            let ws_event = WsEvent::TradeEvent {
                symbol: symbol.as_str().to_string(),
                price: trade.price(),
                quantity: trade.quantity(),
                created_at: trade.created_at(),
            };
            let _ = self.ws_sender.send(ws_event);

            // broadcast updated order (resting order)
            let ws_event = WsEvent::OrderStatusUpdate {
                user_id: resting_order.user_id(),
                order_id: resting_order.id(),
                status: resting_order.status().into(),
                quantity: resting_order.quantity(),
                remaining_quantity: resting_order.remaining_quantity(),
            };
            let _ = self.ws_sender.send(ws_event);
        }

        tx.commit().await?;

        // broadcast updated order (incoming order)
        let ws_event = WsEvent::OrderStatusUpdate {
            user_id: order.user_id(),
            order_id: order.id(),
            status: order.status().into(),
            quantity: order.quantity(),
            remaining_quantity: order.remaining_quantity(),
        };
        let _ = self.ws_sender.send(ws_event);

        // broadcast orderbook snapshot
        let snapshot = self
            .exchange
            .lock()
            .await
            .get_order_book(order.pair_id())?
            .depth(20);

        let ws_event = WsEvent::OrderBookUpdate {
            symbol: symbol.as_str().to_string(),
            snapshot: snapshot,
        };
        let _ = self.ws_sender.send(ws_event);

        let place_order_response = PlaceOrderResponse {
            order_id: order.id(),
            status: order.status().into(),
            filled_quantity: order.quantity() - order.remaining_quantity(),
            remaining_quantity: order.remaining_quantity(),
            trades: final_trades,
        };

        Ok(place_order_response)
    }

    pub async fn cancel_order(
        &self,
        order_id: Uuid,
        user_id: Uuid,
    ) -> Result<OrderResponse, AppError> {
        // verify order exist and belong to logged in user
        let mut order = match db_queries::get_order_by_id(&self.pool, order_id).await? {
            Some(order) => {
                if order.user_id != user_id {
                    return Err(AppError::Forbidden(
                        "Order does not belong to logged in user".to_string(),
                    ));
                }
                order
            }
            None => return Err(AppError::BadRequest("Invalid order ID".to_string())),
        };

        match order.status {
            DBOrderStatus::Cancelled => {
                return Err(AppError::BadRequest(
                    "Order is already cancelled".to_string(),
                ));
            }
            DBOrderStatus::Filled => {
                return Err(AppError::BadRequest(
                    "Filled orders can not be cancelled".to_string(),
                ));
            }
            DBOrderStatus::Open | DBOrderStatus::PartiallyFilled => (),
        }

        self.exchange
            .lock()
            .await
            .cancel_order(order.pair_id, order.id)?;

        // release held balance
        let trading_pair = db_queries::find_trading_pair_by_id(&self.pool, order.pair_id)
            .await?
            .ok_or(AppError::InternalError(
                "Invalid pair ID in order".to_string(),
            ))?;

        let (amount, asset) = match order.side {
            DBOrderSide::Buy => {
                let price = order.price.ok_or(AppError::InternalError(
                    "Limit order should have a price".to_string(),
                ))?;
                let cost = price * order.remaining_quantity;
                (cost, trading_pair.quote_asset)
            }
            DBOrderSide::Sell => (order.remaining_quantity, trading_pair.base_asset),
        };

        db_queries::release(&self.pool, user_id, &asset, amount).await?;

        db_queries::update_order_status(&self.pool, order_id, DBOrderStatus::Cancelled).await?;

        // update status
        order.status = DBOrderStatus::Cancelled;

        // broadcast updated order
        let ws_event = WsEvent::OrderStatusUpdate {
            user_id: order.user_id,
            order_id: order.id,
            status: order.status,
            quantity: order.quantity,
            remaining_quantity: order.remaining_quantity,
        };
        let _ = self.ws_sender.send(ws_event);

        // broadcast orderbook snapshot
        let snapshot = self
            .exchange
            .lock()
            .await
            .get_order_book(order.pair_id)?
            .depth(20);

        let ws_event = WsEvent::OrderBookUpdate {
            symbol: trading_pair.symbol.clone(),
            snapshot: snapshot,
        };
        let _ = self.ws_sender.send(ws_event);

        Ok(OrderResponse::new(order, &trading_pair.symbol))
    }

    pub async fn get_order_by_id(&self, order_id: Uuid) -> Result<(DBOrder, String), AppError> {
        let order = match db_queries::get_order_by_id(&self.pool, order_id).await? {
            Some(order) => order,
            None => return Err(AppError::BadRequest("Invalid order id".to_string())),
        };

        let trading_pair = db_queries::find_trading_pair_by_id(&self.pool, order.pair_id)
            .await?
            .ok_or(AppError::InternalError(
                "Invalid pair ID in order".to_string(),
            ))?;

        Ok((order, trading_pair.symbol))
    }
}
