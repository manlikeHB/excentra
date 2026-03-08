use crate::{
    db::models::order::{DBOrderSide, DBOrderStatus, DBOrderType},
    error::AppError,
};
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
