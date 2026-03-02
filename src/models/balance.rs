use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Balance {
    pub id: Uuid,
    pub user_id: Uuid,
    pub asset: String, // BTC, USDT e.t.c
    pub available: Decimal,
    pub held: Decimal,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
