use uuid::Uuid;

use crate::{db::models::user::UserRole, utils::query_builder::QueryOrder};
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, serde::Deserialize, sqlx::FromRow, serde::Serialize, utoipa::ToSchema)]
pub struct UserSummary {
    pub id: Uuid,
    pub email: String,
    pub role: UserRole,
    pub username: Option<String>,
    pub is_suspended: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[schema(value_type = Vec<BalanceSummary>)]
    pub balances: sqlx::types::Json<Vec<BalanceSummary>>,
}

#[derive(Debug, serde::Deserialize, sqlx::Type, serde::Serialize, utoipa::ToSchema)]
pub struct BalanceSummary {
    pub asset: String,
    pub available: Decimal,
    pub held: Decimal,
}

#[derive(serde::Deserialize)]
pub struct UserSummaryParam {
    pub page: Option<u64>,
    pub limit: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_order")]
    pub order: Option<QueryOrder>,
}

pub fn deserialize_order<'de, D>(deserializer: D) -> Result<Option<QueryOrder>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = Option::<String>::deserialize(deserializer)?;
    match s.as_deref().map(|s| s.to_lowercase()).as_deref() {
        Some("asc") => Ok(Some(QueryOrder::ASC)),
        Some("desc") => Ok(Some(QueryOrder::DESC)),
        None => Ok(None),
        Some(other) => Err(serde::de::Error::custom(format!(
            "unknown status: {}",
            other
        ))),
    }
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct SuspendUserRequest {
    pub suspended: bool,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateUserRoleRequest {
    pub role: UserRole,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct AdminStats {
    pub total_users: i64,
    pub total_trades: i64,
    pub active_ws_connections: u64,
    pub orders_processed: u64,
    pub uptime_seconds: u64,
    pub volume_24h: Vec<PairVolume>,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct PairVolume {
    pub symbol: String,
    pub volume: Decimal,
}
