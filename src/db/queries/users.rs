use crate::db::models::user::{User, UserRole};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_user(
    pool: &PgPool,
    email: &str,
    password_hash: &str,
) -> Result<User, sqlx::Error> {
    sqlx::query_as!(
        User,
        r#"INSERT INTO users (email, password_hash) 
        VALUES ($1, $2) 
        RETURNING id, email, username, password_hash, role as "role: UserRole", created_at, updated_at"#,
        email,
        password_hash
    )
    .fetch_one(pool)
    .await
}

pub async fn find_user_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as!(User, r#"SELECT id, email, username, password_hash, role as "role: UserRole", created_at, updated_at FROM users WHERE email = $1"#, email).fetch_optional(pool).await
}

pub async fn find_user_by_id(pool: &PgPool, user_id: Uuid) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as!(User, r#"SELECT id, email, username, password_hash, role as "role: UserRole", created_at, updated_at FROM users WHERE id = $1"#, user_id).fetch_optional(pool).await
}

pub async fn update_username_and_password(
    pool: &PgPool,
    user_id: Uuid,
    username: &str,
    password_hash: &str,
) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as!(User, r#"UPDATE users SET username = $1, password_hash = $2, updated_at = NOW() WHERE id = $3 RETURNING id, email, username, password_hash, role as "role: UserRole", created_at, updated_at"#, username, password_hash, user_id).fetch_optional(pool).await
}

pub async fn update_password(
    pool: &PgPool,
    user_id: Uuid,
    password_hash: &str,
) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as!(User, r#"UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2 RETURNING id, email, username, password_hash, role as "role: UserRole", created_at, updated_at"#, password_hash, user_id).fetch_optional(pool).await
}

pub async fn update_username(
    pool: &PgPool,
    user_id: Uuid,
    username: &str,
) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as!(User, r#"UPDATE users SET username = $1, updated_at = NOW() WHERE id = $2 RETURNING id, email, username, password_hash, role as "role: UserRole", created_at, updated_at"#, username, user_id).fetch_optional(pool).await
}
