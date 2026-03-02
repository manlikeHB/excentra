use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug)]
pub struct DBTrade {
    id: Uuid,
    pair_id: Uuid,
    buy_order_id: Uuid,
    sell_order_id: Uuid,
    price: Decimal,
    quantity: Decimal,
    created_at: NaiveDateTime,
}
