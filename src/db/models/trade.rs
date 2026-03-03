use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, sqlx::FromRow)]
pub struct DBTrade {
    pub id: Uuid,
    pub pair_id: Uuid,
    pub buy_order_id: Uuid,
    pub sell_order_id: Uuid,
    pub price: Decimal,
    pub quantity: Decimal,
    pub created_at: DateTime<Utc>,
}
