use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::db::models::trading_pairs::DBTradingPair;

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct TradingPairsResponse {
    pub id: Uuid,
    pub base_asset: String,
    pub quote_asset: String,
    pub symbol: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl From<DBTradingPair> for TradingPairsResponse {
    fn from(value: DBTradingPair) -> Self {
        TradingPairsResponse {
            id: value.id,
            base_asset: value.base_asset,
            quote_asset: value.quote_asset,
            symbol: value.symbol,
            is_active: value.is_active,
            created_at: value.created_at,
        }
    }
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct AddTradingPairRequest {
    pub base_asset: String,
    pub quote_asset: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct GetPairParams {
    pub active: Option<bool>,
}
