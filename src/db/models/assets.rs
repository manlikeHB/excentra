use chrono::Utc;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow, serde::Serialize)]
pub struct Asset {
    pub id: Uuid,
    pub symbol: String,
    pub name: String,
    pub decimals: i16,
    pub is_active: bool,
    pub created_at: chrono::DateTime<Utc>,
}
