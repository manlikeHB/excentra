use crate::{
    db::models::order::{DBOrder, DBOrderSide, DBOrderStatus, DBOrderType, OrderWithSymbol},
    error::AppError,
    utils::deserializer::deserialize_order,
    utils::query_builder::QueryOrder,
};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct PlaceOrderRequest {
    pub symbol: String,
    pub side: DBOrderSide,
    pub order_type: DBOrderType,
    pub price: Option<Decimal>,
    pub quantity: Decimal,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema, serde::Deserialize)]
pub struct PlaceOrderResponse {
    pub order_id: Uuid,
    pub status: DBOrderStatus,
    pub filled_quantity: Decimal,
    pub remaining_quantity: Decimal,
    pub trades: Vec<TradeInfo>,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema, serde::Deserialize)]
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
                tracing::warn!("No price in limit order");
                AppError::BadRequest("A limit order should have a price".to_string())
            }
            OrderRequestValidationError::InvalidMarketOrder => {
                tracing::warn!("Price added to market order");
                AppError::BadRequest("A market order does not need a price".to_string())
            }
            OrderRequestValidationError::InvalidPrice => {
                tracing::warn!("Price is or less than zero");
                AppError::BadRequest("Price is or less than zero".to_string())
            }
            OrderRequestValidationError::InvalidQuantity => {
                tracing::warn!("Quantity is or less than zero");
                AppError::BadRequest("Quantity is or less than zero".to_string())
            }
            OrderRequestValidationError::InvalidSymbol => {
                tracing::warn!("Invalid pair symbol used");
                AppError::BadRequest("Symbol should contain '/' e.g 'BTC/USDT'".to_string())
            }
        }
    }
}

#[derive(Debug, sqlx::FromRow, serde::Serialize, utoipa::ToSchema, serde::Deserialize)]
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

impl From<OrderWithSymbol> for OrderResponse {
    fn from(o: OrderWithSymbol) -> Self {
        OrderResponse {
            id: o.id,
            symbol: o.symbol,
            side: o.side,
            order_type: o.order_type,
            price: o.price,
            quantity: o.quantity,
            remaining_quantity: o.remaining_quantity,
            status: o.status,
            created_at: o.created_at,
            updated_at: o.updated_at,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct GetOrdersParams {
    #[serde(default, deserialize_with = "deserialize_status")]
    pub status: Option<Vec<DBOrderStatus>>,
    pub pair: Option<String>,
    pub page: Option<u64>,
    pub limit: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_order")]
    pub order: Option<QueryOrder>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum OneOrMany {
    One(String),
    Many(Vec<String>),
}

pub fn deserialize_status<'de, D>(deserializer: D) -> Result<Option<Vec<DBOrderStatus>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let Some(raw) = Option::<OneOrMany>::deserialize(deserializer)? else {
        return Ok(None);
    };

    let strings = match raw {
        OneOrMany::One(s) => s.split(',').map(|s| s.trim().to_string()).collect(),
        OneOrMany::Many(v) => v,
    };

    let statuses = strings
        .iter()
        .map(|s| match s.to_lowercase().as_str() {
            "open" => Ok(DBOrderStatus::Open),
            "filled" => Ok(DBOrderStatus::Filled),
            "cancelled" => Ok(DBOrderStatus::Cancelled),
            "partially_filled" => Ok(DBOrderStatus::PartiallyFilled),
            other => Err(serde::de::Error::custom(format!(
                "unknown status: {}",
                other
            ))),
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Some(statuses))
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
