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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::order::{DBOrderSide, DBOrderType};
    use rust_decimal_macros::dec;

    fn make_request(
        symbol: &str,
        side: DBOrderSide,
        order_type: DBOrderType,
        price: Option<Decimal>,
        quantity: Decimal,
    ) -> PlaceOrderRequest {
        PlaceOrderRequest {
            symbol: symbol.to_string(),
            side,
            order_type,
            price,
            quantity,
        }
    }

    // ============================================================
    // Symbol validation
    // ============================================================

    #[test]
    fn test_valid_limit_buy() {
        let req = make_request(
            "BTC/USDT",
            DBOrderSide::Buy,
            DBOrderType::Limit,
            Some(dec!(100)),
            dec!(1),
        );
        assert!(req.validate_request().is_ok());
    }

    #[test]
    fn test_invalid_symbol_missing_separator() {
        let req = make_request(
            "BTCUSDT",
            DBOrderSide::Buy,
            DBOrderType::Limit,
            Some(dec!(100)),
            dec!(1),
        );
        assert!(req.validate_request().is_err());
    }

    // ============================================================
    // Limit order validation
    // ============================================================

    #[test]
    fn test_limit_order_missing_price_fails() {
        let req = make_request(
            "BTC/USDT",
            DBOrderSide::Buy,
            DBOrderType::Limit,
            None,
            dec!(1),
        );
        assert!(req.validate_request().is_err());
    }

    #[test]
    fn test_limit_order_zero_price_fails() {
        let req = make_request(
            "BTC/USDT",
            DBOrderSide::Buy,
            DBOrderType::Limit,
            Some(dec!(0)),
            dec!(1),
        );
        assert!(req.validate_request().is_err());
    }

    #[test]
    fn test_limit_order_negative_price_fails() {
        let req = make_request(
            "BTC/USDT",
            DBOrderSide::Buy,
            DBOrderType::Limit,
            Some(dec!(-50)),
            dec!(1),
        );
        assert!(req.validate_request().is_err());
    }

    #[test]
    fn test_limit_order_valid_sell() {
        let req = make_request(
            "ETH/USDT",
            DBOrderSide::Sell,
            DBOrderType::Limit,
            Some(dec!(2000)),
            dec!(0.5),
        );
        assert!(req.validate_request().is_ok());
    }

    // ============================================================
    // Market order validation
    // ============================================================

    #[test]
    fn test_market_order_with_price_fails() {
        let req = make_request(
            "BTC/USDT",
            DBOrderSide::Sell,
            DBOrderType::Market,
            Some(dec!(100)),
            dec!(1),
        );
        assert!(req.validate_request().is_err());
    }

    #[test]
    fn test_market_sell_no_price_valid() {
        let req = make_request(
            "BTC/USDT",
            DBOrderSide::Sell,
            DBOrderType::Market,
            None,
            dec!(1),
        );
        assert!(req.validate_request().is_ok());
    }

    // ============================================================
    // Quantity validation
    // ============================================================

    #[test]
    fn test_zero_quantity_fails() {
        let req = make_request(
            "BTC/USDT",
            DBOrderSide::Buy,
            DBOrderType::Limit,
            Some(dec!(100)),
            dec!(0),
        );
        assert!(req.validate_request().is_err());
    }

    #[test]
    fn test_negative_quantity_fails() {
        let req = make_request(
            "BTC/USDT",
            DBOrderSide::Buy,
            DBOrderType::Limit,
            Some(dec!(100)),
            dec!(-1),
        );
        assert!(req.validate_request().is_err());
    }

    #[test]
    fn test_valid_quantity() {
        let req = make_request(
            "BTC/USDT",
            DBOrderSide::Buy,
            DBOrderType::Limit,
            Some(dec!(100)),
            dec!(0.001),
        );
        assert!(req.validate_request().is_ok());
    }

    // ============================================================
    // Error variants
    // ============================================================

    #[test]
    fn test_missing_price_returns_invalid_limit_order_error() {
        let req = make_request(
            "BTC/USDT",
            DBOrderSide::Buy,
            DBOrderType::Limit,
            None,
            dec!(1),
        );
        assert!(matches!(
            req.validate_request().unwrap_err(),
            OrderRequestValidationError::InvalidLimitOrder
        ));
    }

    #[test]
    fn test_market_with_price_returns_invalid_market_order_error() {
        let req = make_request(
            "BTC/USDT",
            DBOrderSide::Sell,
            DBOrderType::Market,
            Some(dec!(100)),
            dec!(1),
        );
        assert!(matches!(
            req.validate_request().unwrap_err(),
            OrderRequestValidationError::InvalidMarketOrder
        ));
    }

    #[test]
    fn test_invalid_symbol_returns_invalid_symbol_error() {
        let req = make_request(
            "BTCUSDT",
            DBOrderSide::Buy,
            DBOrderType::Limit,
            Some(dec!(100)),
            dec!(1),
        );
        assert!(matches!(
            req.validate_request().unwrap_err(),
            OrderRequestValidationError::InvalidSymbol
        ));
    }

    #[test]
    fn test_zero_price_returns_invalid_price_error() {
        let req = make_request(
            "BTC/USDT",
            DBOrderSide::Buy,
            DBOrderType::Limit,
            Some(dec!(0)),
            dec!(1),
        );
        assert!(matches!(
            req.validate_request().unwrap_err(),
            OrderRequestValidationError::InvalidPrice
        ));
    }

    #[test]
    fn test_zero_quantity_returns_invalid_quantity_error() {
        let req = make_request(
            "BTC/USDT",
            DBOrderSide::Buy,
            DBOrderType::Limit,
            Some(dec!(100)),
            dec!(0),
        );
        assert!(matches!(
            req.validate_request().unwrap_err(),
            OrderRequestValidationError::InvalidQuantity
        ));
    }
}
