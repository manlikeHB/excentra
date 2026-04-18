use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::models::password_reset_tokens::PasswordResetToken;

pub async fn create_reset_token(
    pool: &PgPool,
    user_id: Uuid,
    token_hash: &str,
    expires_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(r#"INSERT INTO password_reset_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)"#, user_id, token_hash, expires_at).execute(pool).await?;
    Ok(())
}

pub async fn get_valid_reset_token(
    pool: &PgPool,
    token_hash: &str,
) -> Result<Option<PasswordResetToken>, sqlx::Error> {
    sqlx::query_as!(PasswordResetToken, r#"SELECT id, user_id, token_hash, expires_at, used_at, created_at FROM password_reset_tokens WHERE token_hash = $1 AND used_at IS NULL AND expires_at > NOW()"#, token_hash).fetch_optional(pool).await
}

pub async fn mark_reset_token_used(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE password_reset_tokens SET used_at = NOW() WHERE id = $1"#,
        id
    )
    .execute(pool)
    .await?;
    Ok(())
}
