use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
pub enum DBOrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy)]
pub enum DBOrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone, Copy)]
pub enum DBOrderStatus {
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
}

#[derive(Debug, Clone, Copy)]
pub struct DBOrder {
    id: Uuid,
    user_id: Uuid,
    pair_id: Uuid,
    side: DBOrderSide,
    order_type: DBOrderType,
    price: Option<Decimal>,
    quantity: Decimal,
    remaining_quantity: Decimal,
    status: DBOrderStatus,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}
