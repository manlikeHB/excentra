use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::db::models::order::{DBOrder, DBOrderSide, DBOrderStatus, DBOrderType};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Order {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        user_id: Uuid,
        pair_id: Uuid,
        side: OrderSide,
        order_type: OrderType,
        price: Option<Decimal>,
        quantity: Decimal,
        remaining_quantity: Decimal,
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
            status: OrderStatus::Open,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
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
        self.updated_at = chrono::Utc::now();
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

    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

impl From<DBOrder> for Order {
    fn from(db_order: DBOrder) -> Self {
        Order {
            id: db_order.id,
            user_id: db_order.user_id,
            pair_id: db_order.pair_id,
            side: db_order.side.into(),
            order_type: db_order.order_type.into(),
            price: db_order.price,
            quantity: db_order.quantity,
            remaining_quantity: db_order.remaining_quantity,
            status: db_order.status.into(),
            created_at: db_order.created_at,
            updated_at: db_order.updated_at,
        }
    }
}

impl From<DBOrderSide> for OrderSide {
    fn from(value: DBOrderSide) -> Self {
        match value {
            DBOrderSide::Buy => OrderSide::Buy,
            DBOrderSide::Sell => OrderSide::Sell,
        }
    }
}

impl From<DBOrderStatus> for OrderStatus {
    fn from(value: DBOrderStatus) -> Self {
        match value {
            DBOrderStatus::Cancelled => OrderStatus::Cancelled,
            DBOrderStatus::Filled => OrderStatus::Filled,
            DBOrderStatus::Open => OrderStatus::Open,
            DBOrderStatus::PartiallyFilled => OrderStatus::PartiallyFilled,
        }
    }
}

impl From<DBOrderType> for OrderType {
    fn from(value: DBOrderType) -> Self {
        match value {
            DBOrderType::Limit => OrderType::Limit,
            DBOrderType::Market => OrderType::Market,
        }
    }
}
