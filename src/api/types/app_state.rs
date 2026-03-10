use crate::services::orders::OrderService;
use sqlx::PgPool;

pub struct AppState {
    pub pool: PgPool,
    pub order_service: OrderService,
    pub jwt_secret: String,
}
