use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct DBRefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub used_at: Option<chrono::DateTime<chrono::Utc>>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
