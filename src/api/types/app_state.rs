use crate::engine::exchange::Exchange;
use sqlx::PgPool;
use tokio::sync::Mutex;

pub struct AppState {
    pub pool: PgPool,
    pub exchange: Mutex<Exchange>,
    pub jwt_secret: String,
}
