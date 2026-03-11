use crate::{
    db::models::order::{DBOrder, DBOrderSide, DBOrderStatus, DBOrderType},
    error::AppError,
};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, serde::Deserialize)]
pub struct PlaceOrderRequest {
    pub symbol: String,
    pub side: DBOrderSide,
    pub order_type: DBOrderType,
    pub price: Option<Decimal>,
    pub quantity: Decimal,
}

#[derive(Debug, serde::Serialize)]
pub struct PlaceOrderResponse {
    pub order_id: Uuid,
    pub status: DBOrderStatus,
    pub filled_quantity: Decimal,
    pub remaining_quantity: Decimal,
    pub trades: Vec<TradeInfo>,
}

#[derive(Debug, serde::Serialize)]
pub struct TradeInfo {
    pub price: Decimal,
    pub quantity: Decimal,
}

impl PlaceOrderRequest {
    pub fn validate_request(&self) -> Result<bool, OrderRequestValidationError> {
        if !self.symbol.contains("/") {
            return Err(OrderRequestValidationError::InvalidSymbol);
        };

        match self.order_type {
            DBOrderType::Limit => {
                match self.price {
                    Some(p) => {
                        if p <= Decimal::ZERO {
                            return Err(OrderRequestValidationError::InvalidPrice);
                        };
                    }
                    None => return Err(OrderRequestValidationError::InvalidLimitOrder),
                };
            }
            DBOrderType::Market => {
                if self.price.is_some() {
                    return Err(OrderRequestValidationError::InvalidMarketOrder);
                }
            }
        }

        if self.quantity <= Decimal::ZERO {
            return Err(OrderRequestValidationError::InvalidQuantity);
        }

        Ok(true)
    }
}

pub enum OrderRequestValidationError {
    InvalidSymbol,
    InvalidPrice,
    InvalidLimitOrder,
    InvalidMarketOrder,
    InvalidQuantity,
}

impl From<OrderRequestValidationError> for AppError {
    fn from(value: OrderRequestValidationError) -> Self {
        match value {
            OrderRequestValidationError::InvalidLimitOrder => {
                AppError::BadRequest("A limit order should have a price".to_string())
            }
            OrderRequestValidationError::InvalidMarketOrder => {
                AppError::BadRequest("A market order does not need a price".to_string())
            }
            OrderRequestValidationError::InvalidPrice => {
                AppError::BadRequest("Price is or less than zero".to_string())
            }
            OrderRequestValidationError::InvalidQuantity => {
                AppError::BadRequest("Quantity is or less than zero".to_string())
            }
            OrderRequestValidationError::InvalidSymbol => {
                AppError::BadRequest("Symbol should contain '/' e.g 'BTC/USDT'".to_string())
            }
        }
    }
}

#[derive(Debug, sqlx::FromRow, serde::Serialize)]
pub struct OrderResponse {
    pub id: Uuid,
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

impl OrderResponse {
    pub fn new(order: DBOrder, symbol: &str) -> Self {
        OrderResponse {
            id: order.id,
            symbol: symbol.to_string(),
            side: order.side,
            order_type: order.order_type,
            price: order.price,
            quantity: order.quantity,
            remaining_quantity: order.remaining_quantity,
            status: order.status,
            created_at: order.created_at,
            updated_at: order.updated_at,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct CancelOrderRequest {
    pub order_id: Uuid,
}
