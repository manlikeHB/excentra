use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
pub enum OrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone)]
pub enum OrderStatus {
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
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
    updated_at: NaiveDateTime,
}

impl Order {
    pub fn new(
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
        updated_at: NaiveDateTime,
    ) -> Self {
        Order {
            id,
            user_id,
            pair_id,
            side,
            order_type,
            price,
            quantity,
            remaining_quantity,
            status,
            created_at,
            updated_at,
        }
    }

    pub fn price(&self) -> Option<Decimal> {
        self.price
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn side(&self) -> &OrderSide {
        &self.side
    }
}
