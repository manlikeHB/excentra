use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy)]
pub enum OrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone, Copy)]
pub enum OrderStatus {
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
}

#[derive(Debug, Clone, Copy)]
pub struct Order {
    id: Uuid,
    user_id: Uuid,
    pair_id: Uuid,
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
        user_id: Uuid,
        pair_id: Uuid,
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

    pub fn quantity(&self) -> Decimal {
        self.quantity
    }

    pub fn remaining_quantity(&self) -> Decimal {
        self.remaining_quantity
    }

    pub fn pair_id(&self) -> Uuid {
        self.pair_id
    }

    pub fn reduce_quantity(&mut self, amount: Decimal) {
        self.remaining_quantity -= amount;
        if self.remaining_quantity == Decimal::ZERO {
            self.status = OrderStatus::Filled;
        } else {
            self.status = OrderStatus::PartiallyFilled;
        }
        self.updated_at = chrono::Utc::now().naive_utc();
    }

    pub fn status(&self) -> OrderStatus {
        self.status
    }

    pub fn order_type(&self) -> OrderType {
        self.order_type
    }

    pub fn set_status(&mut self, status: OrderStatus) {
        self.status = status
    }
}
