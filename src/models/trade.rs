use uuid::Uuid;
use rust_decimal::Decimal;
use chrono::NaiveDateTime;

#[derive(Debug)]
pub struct Trade {
    id: Uuid,
    pair_id: Uuid,
    buy_order_id: Uuid,
    sell_order_id: Uuid,
    price: Decimal,
    quantity: Decimal,
    created_at: NaiveDateTime,
}