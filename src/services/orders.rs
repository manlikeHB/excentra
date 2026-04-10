use rust_decimal::Decimal;
use std::sync::{Arc, atomic::AtomicU64};
use tokio::sync::{Mutex, broadcast};

use sqlx::{PgPool, QueryBuilder};
use uuid::Uuid;

use crate::{
    api::types::order::{
        GetOrdersParams, OrderRequestValidationError, OrderResponse, PlaceOrderRequest,
        PlaceOrderResponse, TradeInfo,
    },
    db::models::{
        order::{DBOrder, DBOrderStatus, OrderWithSymbol},
        trading_pairs::DBTradingPair,
    },
    engine::{
        exchange::Exchange,
        matcher::MatchResult,
        models::order::{Order, OrderSide, OrderType},
    },
    error::AppError,
    types::asset_symbol::AssetSymbol,
    utils::query_builder::{self},
    ws::messages::WsEvent,
};

use crate::db::{
    models::order::{DBOrderSide, DBOrderType},
    queries::{self as db_queries},
};
use std::sync::atomic::Ordering;

pub struct OrderService {
    pool: PgPool,
    exchange: Arc<Mutex<Exchange>>,
    ws_sender: broadcast::Sender<WsEvent>,
    orders_processed: Arc<AtomicU64>,
}

impl OrderService {
    pub fn new(
        pool: PgPool,
        exchange: Arc<Mutex<Exchange>>,
        ws_sender: broadcast::Sender<WsEvent>,
        orders_processed: Arc<AtomicU64>,
    ) -> Self {
        OrderService {
            pool,
            exchange,
            ws_sender,
            orders_processed,
        }
    }

    pub fn orders_processed(&self) -> u64 {
        self.orders_processed.load(Ordering::Relaxed)
    }

    pub async fn place_order(
        &self,
        user_id: Uuid,
        body: PlaceOrderRequest,
    ) -> Result<PlaceOrderResponse, AppError> {
        let asset_symbol = AssetSymbol::new(&body.symbol)?;
        // get trading pair
        let trading_pair = self.get_trading_pair(asset_symbol.as_str()).await?;

        // Self-Trade Prevention (STP): reject incoming limit orders that would cross
        // the user's own resting orders. Market orders are exempt — a user may
        // intentionally hold a resting limit order while placing a market order
        // for a different purpose. For market orders, wash trading is mitigated
        // at the engine level via break 'outer in the matching loop.
        if body.order_type == DBOrderType::Limit {
            let price = match body.price {
                Some(p) => p,
                None => {
                    return Err(AppError::BadRequest(
                        "A limit order should have a price".to_string(),
                    ));
                }
            };

            let has_crossing = db_queries::has_crossing_order(
                &self.pool,
                user_id,
                trading_pair.id,
                &body.side,
                price,
            )
            .await?;

            if has_crossing {
                return Err(AppError::BadRequest(
                    "Order would match against your own resting order".to_string(),
                ));
            }
        }

        // get quote asset
        let quote_asset =
            match db_queries::find_asset_by_symbol(&self.pool, asset_symbol.quote_asset()).await? {
                Some(a) => a,
                None => {
                    return Err(AppError::InternalError(format!(
                        "Invalid Asset: {} symbol in trading pair",
                        &trading_pair.quote_asset
                    )));
                }
            };

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

                tracing::info!(order_id = %order.id(), user_id = %user_id, amount_released = %amount, "Balance released");

                return Err(e.into());
            }
        };

        // persist order in DB
        let place_order_response = self
            .persist_order_and_trades(
                order,
                &match_result,
                asset_symbol,
                quote_asset.decimals as u32,
            )
            .await?;

        tracing::info!(order_id = %order.id(), user_id = %order.user_id(), pair = %order.pair_id(), side = ?order.side(), order_type = ?order.order_type(), quantity = %order.quantity(), trades_count = %match_result.trades().len(), "Order placed");

        self.orders_processed.fetch_add(1, Ordering::Relaxed);

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

    // quantity param in PlaceOrderRequest represents the quantity to buy or sell, but
    // for a market buy it represents the budget (max spend for the quote asset)
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
                        let price = match body.price {
                            Some(p) => p,
                            None => {
                                return Err(OrderRequestValidationError::InvalidLimitOrder.into());
                            }
                        };
                        let cost = price * body.quantity;

                        self.check_balance_and_hold_asset(user_id, &trading_pair.quote_asset, cost)
                            .await?
                    }
                    DBOrderSide::Sell => {
                        // check base balance when selling for some quote asset
                        self.check_balance_and_hold_asset(
                            user_id,
                            &trading_pair.base_asset,
                            body.quantity,
                        )
                        .await?
                    }
                }
            }
            DBOrderType::Market => {
                match body.side {
                    // check quote balance against what trader is willing to spend (budget)
                    DBOrderSide::Buy => {
                        self.check_balance_and_hold_asset(
                            user_id,
                            &trading_pair.quote_asset,
                            body.quantity,
                        )
                        .await?
                    }
                    DBOrderSide::Sell => {
                        // check base balance when selling for some quote asset
                        self.check_balance_and_hold_asset(
                            user_id,
                            &trading_pair.base_asset,
                            body.quantity,
                        )
                        .await?
                    }
                }
            }
        };

        Ok((asset.clone(), amount))
    }

    async fn check_balance_and_hold_asset(
        &self,
        user_id: Uuid,
        asset: &str,
        amount: Decimal,
    ) -> Result<(String, Decimal), AppError> {
        match db_queries::get_balance(&self.pool, user_id, asset).await? {
            Some(bal) => {
                if bal.available < amount {
                    return Err(AppError::Unprocessable(format!(
                        "Insufficient available {} balance",
                        bal.asset
                    )));
                } else {
                    // hold base asset
                    db_queries::hold(&self.pool, user_id, asset, amount).await?;
                    tracing::info!(user_id = %user_id, amount_held = %amount, "Balance held");
                    Ok((asset.to_string(), amount))
                }
            }
            None => {
                return Err(AppError::Unprocessable(format!(
                    "Insufficient available {} balance",
                    asset
                )));
            }
        }
    }

    async fn persist_order_and_trades(
        &self,
        order: Order,
        match_result: &MatchResult,
        symbol: AssetSymbol,
        quote_precision: u32,
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
                symbol.base_asset(),
                symbol.quote_asset(),
                trade.quantity(),
                trade.price(),
                quote_precision,
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

            tracing::info!(trade_id = %trade.id(), pair = %trade.pair_id(), price = %trade.price(), quantity = %trade.quantity(), "Trade executed");
        }

        // release unspent budget for a market buy order
        match (order.order_type(), order.side()) {
            (OrderType::Market, OrderSide::Buy) => {
                let unspent = order.remaining_quantity();
                if unspent > Decimal::ZERO {
                    db_queries::release(&mut *tx, order.user_id(), &symbol.quote_asset(), unspent)
                        .await?;

                    tracing::info!(order_id = %order.id(), user_id = %order.user_id(), amount = %unspent, "Release unspent balance");
                }
            }
            (_, _) => (),
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

        // filled quantity has to be calculated for market buy orders since remaining_quantity field held the quote asset budget
        let filled_quantity = match (order.order_type(), order.side()) {
            (OrderType::Market, OrderSide::Buy) => final_trades.iter().map(|t| t.quantity).sum(),
            _ => order.quantity() - order.remaining_quantity(),
        };

        // remaining_quantity for market buy order is set to zero since it's either totally
        // filled or remaining budget is returned to the buyer
        let remaining_quantity = match (order.order_type(), order.side()) {
            (OrderType::Market, OrderSide::Buy) => Decimal::ZERO,
            _ => order.remaining_quantity(),
        };

        let place_order_response = PlaceOrderResponse {
            order_id: order.id(),
            status: order.status().into(),
            filled_quantity,
            remaining_quantity,
            trades: final_trades,
        };

        tracing::info!(
            order_id = %order.id(),
            status = ?order.status(),
            remaining_quantity = %remaining_quantity,
            trades_count = match_result.trades().len(),
            "Order matching complete"
        );

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

        let mut tx = self.pool.begin().await?;
        db_queries::release(&mut *tx, user_id, &asset, amount).await?;

        db_queries::update_order_status(&mut *tx, order_id, DBOrderStatus::Cancelled).await?;

        tx.commit().await?;

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

        tracing::info!(order_id = %order.id, user_id = %order.user_id, status = ?order.status, "order canceled");

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

    pub async fn get_orders(
        &self,
        user_id: Uuid,
        params: &GetOrdersParams,
    ) -> Result<(Vec<OrderWithSymbol>, i64), AppError> {
        // build order query
        let mut order_builder = sqlx::QueryBuilder::new(
            "SELECT o.id, tp.symbol, o.side, o.user_id, o.pair_id, o.order_type, o.price, 
                o.quantity, o.remaining_quantity, o.status, o.created_at, o.updated_at
                FROM orders o
                JOIN trading_pairs tp ON o.pair_id = tp.id
                WHERE o.user_id = ",
        );
        order_builder.push_bind(user_id);
        query_builder::apply_status_filter(&mut order_builder, params.status);
        query_builder::apply_pair_filter(
            &self.pool,
            &mut order_builder,
            params.pair.as_deref(),
            "o",
        )
        .await?;
        query_builder::apply_pagination(
            &mut order_builder,
            params.page,
            params.limit,
            "o",
            params.order,
        );

        // build count order query
        let mut count_builder = QueryBuilder::new(
            "SELECT COUNT(*) FROM orders o
                JOIN trading_pairs tp ON o.pair_id = tp.id
                WHERE o.user_id = ",
        );
        count_builder.push_bind(user_id);
        query_builder::apply_status_filter(&mut count_builder, params.status);
        query_builder::apply_pair_filter(
            &self.pool,
            &mut count_builder,
            params.pair.as_deref(),
            "o",
        )
        .await?;

        // execute queries
        let orders: Vec<OrderWithSymbol> =
            order_builder.build_query_as().fetch_all(&self.pool).await?;
        let count: i64 = count_builder
            .build_query_scalar()
            .fetch_one(&self.pool)
            .await?;

        tracing::info!(user_id = %user_id, total = count, "Orders fetched");

        Ok((orders, count))
    }
}
