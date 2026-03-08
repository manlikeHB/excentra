use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
pub struct Trade {
    id: Uuid,
    pair_id: Uuid,
    buyer_id: Uuid,
    seller_id: Uuid,
    buy_order_id: Uuid,
    sell_order_id: Uuid,
    price: Decimal,
    quantity: Decimal,
    created_at: DateTime<Utc>,
}

impl Trade {
    pub fn new(
        id: Uuid,
        pair_id: Uuid,
        buyer_id: Uuid,
        seller_id: Uuid,
        buy_order_id: Uuid,
        sell_order_id: Uuid,
        price: Decimal,
        quantity: Decimal,
        created_at: DateTime<Utc>,
    ) -> Self {
        Trade {
            id,
            pair_id,
            buyer_id,
            seller_id,
            buy_order_id,
            sell_order_id,
            price,
            quantity,
            created_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn pair_id(&self) -> Uuid {
        self.pair_id
    }

    pub fn price(&self) -> Decimal {
        self.price
    }

    pub fn quantity(&self) -> Decimal {
        self.quantity
    }

    pub fn buy_order_id(&self) -> Uuid {
        self.buy_order_id
    }
    pub fn sell_order_id(&self) -> Uuid {
        self.sell_order_id
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn buyer_id(&self) -> Uuid {
        self.buyer_id
    }

    pub fn seller_id(&self) -> Uuid {
        self.seller_id
    }
}
