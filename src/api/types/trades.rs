use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::db::models::{
    order::DBOrderSide,
    trade::{DBTrade, TradeWithSymbolAndSide},
};

#[derive(Debug, serde::Serialize)]
pub struct TradeResponse {
    pub id: Uuid,
    pub symbol: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub created_at: DateTime<Utc>,
}

impl TradeResponse {
    pub fn new(t: DBTrade, symbol: String) -> Self {
        TradeResponse {
            id: t.id,
            symbol,
            price: t.price,
            quantity: t.quantity,
            created_at: t.created_at,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct TradeParams {
    pub limit: Option<u64>,
    pub pair: Option<String>,
    pub page: Option<u64>,
}

#[derive(Debug, serde::Serialize)]
pub struct UserTradeResponse {
    pub id: Uuid,
    pub symbol: String,
    pub side: DBOrderSide,
    pub price: Decimal,
    pub quantity: Decimal,
    pub total: Decimal,
    pub created_at: DateTime<Utc>,
}

impl From<TradeWithSymbolAndSide> for UserTradeResponse {
    fn from(t: TradeWithSymbolAndSide) -> Self {
        UserTradeResponse {
            id: t.id,
            symbol: t.symbol,
            side: t.side,
            price: t.price,
            quantity: t.quantity,
            total: t.price * t.quantity,
            created_at: t.created_at,
        }
    }
}
