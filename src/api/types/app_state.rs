use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    engine::exchange::Exchange,
    services::{orders::OrderService, trading_pair::TradingPairService},
};
use sqlx::PgPool;

pub struct AppState {
    pub pool: PgPool,
    pub order_service: OrderService,
    pub trading_pair_service: TradingPairService,
    pub exchange: Arc<Mutex<Exchange>>,
    pub jwt_secret: String,
}
