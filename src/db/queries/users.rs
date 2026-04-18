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
        RETURNING id, email, username, password_hash, role as "role: UserRole", created_at, updated_at, is_suspended"#,
        email,
        password_hash
    )
    .fetch_one(pool)
    .await
}

pub async fn find_user_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as!(User, r#"SELECT id, email, username, password_hash, role as "role: UserRole", created_at, updated_at, is_suspended FROM users WHERE email = $1"#, email).fetch_optional(pool).await
}

pub async fn find_user_by_id(pool: &PgPool, user_id: Uuid) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as!(User, r#"SELECT id, email, username, password_hash, role as "role: UserRole", created_at, updated_at, is_suspended FROM users WHERE id = $1"#, user_id).fetch_optional(pool).await
}

pub async fn update_username_and_password(
    pool: &PgPool,
    user_id: Uuid,
    username: &str,
    password_hash: &str,
) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as!(User, r#"UPDATE users SET username = $1, password_hash = $2, updated_at = NOW() WHERE id = $3 RETURNING id, email, username, password_hash, role as "role: UserRole", created_at, updated_at, is_suspended"#, username, password_hash, user_id).fetch_optional(pool).await
}

pub async fn update_password(
    pool: &PgPool,
    user_id: Uuid,
    password_hash: &str,
) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as!(User, r#"UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2 RETURNING id, email, username, password_hash, role as "role: UserRole", created_at, updated_at, is_suspended"#, password_hash, user_id).fetch_optional(pool).await
}

pub async fn update_username(
    pool: &PgPool,
    user_id: Uuid,
    username: &str,
) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as!(User, r#"UPDATE users SET username = $1, updated_at = NOW() WHERE id = $2 RETURNING id, email, username, password_hash, role as "role: UserRole", created_at, updated_at, is_suspended"#, username, user_id).fetch_optional(pool).await
}

pub async fn count_users(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let count = sqlx::query_scalar!("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?
        .unwrap_or(0i64);
    Ok(count)
}

pub async fn suspend_user(
    pool: &PgPool,
    user_id: Uuid,
    is_suspended: bool,
) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as!(User, r#"UPDATE users SET is_suspended = $1, updated_at = NOW() WHERE id = $2 RETURNING id, email, username, password_hash, role as "role: UserRole", created_at, updated_at, is_suspended"#, is_suspended, user_id).fetch_optional(pool).await
}

pub async fn update_role(
    pool: &PgPool,
    user_id: Uuid,
    role: UserRole,
) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as!(User, r#"UPDATE users SET role = $1, updated_at = NOW() WHERE id = $2 RETURNING id, email, username, password_hash, role as "role: UserRole", created_at, updated_at, is_suspended"#, role as UserRole, user_id).fetch_optional(pool).await
}
