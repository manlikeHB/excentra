use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::db::models::trade::DBTrade;

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
