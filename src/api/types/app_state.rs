use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    engine::exchange::Exchange,
    services::{
        assets::AssetService, orderbook::OrderBookService, orders::OrderService,
        trades::TradeService, trading_pair::TradingPairService,
    },
};
use sqlx::PgPool;

pub struct AppState {
    pub pool: PgPool,
    pub order_service: OrderService,
    pub trading_pair_service: TradingPairService,
    pub trade_service: TradeService,
    pub asset_service: AssetService,
    pub order_book_service: OrderBookService,
    pub exchange: Arc<Mutex<Exchange>>,
    pub jwt_secret: String,
}
