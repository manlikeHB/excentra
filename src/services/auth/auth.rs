use crate::db::models::user::User;
use crate::db::queries as db_queries;
use crate::error::AppError;
use crate::services::auth::utils::{create_token, hash_password, verify_password};
use sqlx::PgPool;

pub struct AuthService {
    pub pool: PgPool,
    pub jwt_secret: String,
}

impl AuthService {
    pub fn new(pool: PgPool, jwt_secret: String) -> Self {
        AuthService { pool, jwt_secret }
    }

    pub async fn register_user(&self, email: &str, password: &str) -> Result<User, AppError> {
        let password_hash = hash_password(password)?;

        let new_user = db_queries::create_user(&self.pool, email, &password_hash).await?;
        Ok(new_user)
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<String, AppError> {
        let user = match db_queries::find_by_email(&self.pool, email).await? {
            Some(user) => user,
            None => {
                return Err(AppError::Unauthorized(
                    "Invalid email or password".to_string(),
                ));
            }
        };

        match verify_password(password, &user.password_hash)? {
            true => (),
            false => {
                tracing::warn!(email = %email, "Failed login attempt");
                return Err(AppError::Unauthorized(
                    "Invalid email or password".to_string(),
                ));
            }
        };

        Ok(create_token(user.id, user.role, &self.jwt_secret)?)
    }
}
