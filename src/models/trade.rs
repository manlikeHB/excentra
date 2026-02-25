use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use uuid::Uuid;

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

impl Trade {
    pub fn new(
        id: Uuid,
        pair_id: Uuid,
        buy_order_id: Uuid,
        sell_order_id: Uuid,
        price: Decimal,
        quantity: Decimal,
        created_at: NaiveDateTime,
    ) -> Self {
        Trade {
            id,
            pair_id,
            buy_order_id,
            sell_order_id,
            price,
            quantity,
            created_at,
        }
    }
}
