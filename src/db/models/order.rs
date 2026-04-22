use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::engine::models::order::{Order, OrderSide, OrderStatus, OrderType};

#[derive(
    Debug,
    Clone,
    Copy,
    sqlx::Type,
    serde::Deserialize,
    serde::Serialize,
    utoipa::ToSchema,
    PartialEq,
    Eq,
)]
#[sqlx(type_name = "order_side", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum DBOrderSide {
    Buy,
    Sell,
}

#[derive(
    Debug,
    Clone,
    Copy,
    sqlx::Type,
    serde::Deserialize,
    serde::Serialize,
    PartialEq,
    Eq,
    utoipa::ToSchema,
)]
#[sqlx(type_name = "order_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum DBOrderType {
    Market,
    Limit,
}

#[derive(
    Debug,
    Clone,
    Copy,
    sqlx::Type,
    serde::Deserialize,
    serde::Serialize,
    utoipa::ToSchema,
    PartialEq,
    Eq,
)]
#[sqlx(type_name = "order_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum DBOrderStatus {
    Open,
    #[sqlx(rename = "partially_filled")]
    #[serde(rename = "partially_filled")]
    PartiallyFilled,
    Filled,
    Cancelled,
}

#[derive(Debug, Clone, Copy, sqlx::FromRow)]
pub struct DBOrder {
    pub id: Uuid,
    pub user_id: Uuid,
    pub pair_id: Uuid,
    pub side: DBOrderSide,
    pub order_type: DBOrderType,
    pub price: Option<Decimal>,
    pub quantity: Decimal,
    pub remaining_quantity: Decimal,
    pub status: DBOrderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Order> for DBOrder {
    fn from(order: Order) -> Self {
        DBOrder {
            id: order.id(),
            user_id: order.user_id(),
            pair_id: order.pair_id(),
            side: (*order.side()).into(),
            order_type: order.order_type().into(),
            price: order.price(),
            quantity: order.quantity(),
            remaining_quantity: order.remaining_quantity(),
            status: order.status().into(),
            created_at: order.created_at(),
            updated_at: order.updated_at(),
        }
    }
}

impl From<OrderSide> for DBOrderSide {
    fn from(value: OrderSide) -> Self {
        match value {
            OrderSide::Buy => DBOrderSide::Buy,
            OrderSide::Sell => DBOrderSide::Sell,
        }
    }
}

impl From<OrderStatus> for DBOrderStatus {
    fn from(value: OrderStatus) -> Self {
        match value {
            OrderStatus::Cancelled => DBOrderStatus::Cancelled,
            OrderStatus::Filled => DBOrderStatus::Filled,
            OrderStatus::Open => DBOrderStatus::Open,
            OrderStatus::PartiallyFilled => DBOrderStatus::PartiallyFilled,
        }
    }
}

impl From<OrderType> for DBOrderType {
    fn from(value: OrderType) -> Self {
        match value {
            OrderType::Limit => DBOrderType::Limit,
            OrderType::Market => DBOrderType::Market,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct OrderWithSymbol {
    pub id: Uuid,
    pub user_id: Uuid,
    pub pair_id: Uuid,
    pub symbol: String,
    pub side: DBOrderSide,
    pub order_type: DBOrderType,
    pub price: Option<Decimal>,
    pub quantity: Decimal,
    pub remaining_quantity: Decimal,
    pub status: DBOrderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
