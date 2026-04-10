use uuid::Uuid;

use crate::{db::models::user::UserRole, utils::query_builder::QueryOrder};
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, serde::Deserialize, sqlx::FromRow, serde::Serialize)]
pub struct UserSummary {
    pub id: Uuid,
    pub email: String,
    pub role: UserRole,
    pub username: Option<String>,
    pub is_suspended: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub balances: sqlx::types::Json<Vec<BalanceSummary>>,
}

#[derive(Debug, serde::Deserialize, sqlx::Type, serde::Serialize)]
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

#[derive(Debug, serde::Deserialize)]
pub struct SuspendUserRequest {
    pub suspended: bool,
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateUserRoleRequest {
    pub role: UserRole,
}
