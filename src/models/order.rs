use rust_decimal::Decimal;
use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum OrderSide {
    Buy,
    Sell
}

#[derive(Debug, Clone)]
pub enum OrderType {
    Market,
    Limit
}

#[derive(Debug, Clone)]
pub enum OrderStatus {
    Open,
    PartiallyFilled,
    Filled,
    Cancelled
}

#[derive(Debug)]
pub struct Order {
    id: Uuid,
    user_id: u64,
    pair_id: u64,
    side: OrderSide,
    order_type: OrderType,
    price: Option<Decimal>,
    quantity: Decimal,
    remaining_quantity: Decimal,
    status: OrderStatus,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime
}