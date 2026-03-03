use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, sqlx::Type)]
#[sqlx(type_name = "order_side", rename_all = "lowercase")]
pub enum DBOrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, sqlx::Type)]
#[sqlx(type_name = "order_type", rename_all = "lowercase")]
pub enum DBOrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone, Copy, sqlx::Type)]
#[sqlx(type_name = "order_status", rename_all = "lowercase")]
pub enum DBOrderStatus {
    Open,
    #[sqlx(rename = "partially_filled")]
    PartiallyFilled,
    Filled,
    Cancelled,
}

#[derive(Debug, Clone, Copy, sqlx::FromRow)]
pub struct DBOrder {
    pub id: Uuid,
    pub user_id: Uuid,
    pub pair_id: Uuid,
    pub side: DBOrderSide,
    pub order_type: DBOrderType,
    pub price: Option<Decimal>,
    pub quantity: Decimal,
    pub remaining_quantity: Decimal,
    pub status: DBOrderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
