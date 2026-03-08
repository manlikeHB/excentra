use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct DBTradingPair {
    pub id: Uuid,
    pub base_asset: String,
    pub quote_asset: String,
    pub symbol: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}
