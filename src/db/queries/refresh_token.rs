use sqlx::PgPool;
use uuid::Uuid;

use crate::db::models::refresh_token::DBRefreshToken;

pub async fn create_refresh_token(
    pool: &PgPool,
    user_id: Uuid,
    token_hash: &str,
    expires_at: chrono::DateTime<chrono::Utc>,
) -> Result<DBRefreshToken, sqlx::Error> {
    sqlx::query_as!(DBRefreshToken, r#"INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3) RETURNING id, user_id, token_hash, expires_at, used_at, created_at"#, user_id, token_hash, expires_at).fetch_one(pool).await
}

pub async fn find_refresh_token(
    pool: &PgPool,
    token_hash: &str,
) -> Result<Option<DBRefreshToken>, sqlx::Error> {
    sqlx::query_as!(DBRefreshToken, r#"SELECT id, user_id, token_hash, used_at, expires_at, created_at FROM refresh_tokens WHERE token_hash = $1"#, token_hash).fetch_optional(pool).await
}

pub async fn delete_refresh_token(pool: &PgPool, token_hash: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"DELETE FROM refresh_tokens WHERE token_hash = $1"#,
        token_hash
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_all_user_refresh_tokens(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(r#"DELETE FROM refresh_tokens WHERE user_id = $1"#, user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn mark_refresh_token_used(pool: &PgPool, token_hash: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE refresh_tokens SET used_at = NOW() WHERE token_hash = $1"#,
        token_hash
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_all_stale_refresh_tokens(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(r#"DELETE FROM refresh_tokens WHERE expires_at < NOW()"#)
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}
