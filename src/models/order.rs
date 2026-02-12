use rust_decimal::Decimal;
use chrono::NaiveDateTime;

#[derive(Debug)]
pub enum OrderSide {
    Buy,
    Sell
}

#[derive(Debug)]
pub enum OrderType {
    Market,
    Limit
}

#[derive(Debug)]
pub enum OrderStatus {
    Open,
    PartiallyFilled,
    Filled,
    Cancelled
}

#[derive(Debug)]
pub struct Order {
    pub id: u64,
    pub user_id: u64,
    pub pair_id: u64,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub price: Option<Decimal>,
    pub quantity: Decimal,
    pub remaining_quantity: Decimal,
    pub status: OrderStatus,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime
}