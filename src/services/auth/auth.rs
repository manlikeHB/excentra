use crate::db::queries as db_queries;
use crate::error::AppError;
use crate::services::auth::utils::{
    create_token, generate_refresh_token, hash_password, hash_refresh_token, verify_password,
};
use sqlx::PgPool;

pub struct AuthService {
    pub pool: PgPool,
    pub jwt_secret: String,
}

impl AuthService {
    pub fn new(pool: PgPool, jwt_secret: String) -> Self {
        AuthService { pool, jwt_secret }
    }

    pub async fn register_user(
        &self,
        email: &str,
        password: &str,
    ) -> Result<(String, String), AppError> {
        let password_hash = hash_password(password)?;

        db_queries::create_user(&self.pool, email, &password_hash).await?;

        // auto login after registration
        Ok(self.login(email, password).await?)
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<(String, String), AppError> {
        let user = match db_queries::find_user_by_email(&self.pool, email).await? {
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

        let access_token = create_token(user.id, user.role, &self.jwt_secret)?;
        let refresh_token = generate_refresh_token();
        let token_hash = hash_refresh_token(&refresh_token);
        let expires_at = chrono::Utc::now() + chrono::Duration::days(7);
        db_queries::create_refresh_token(&self.pool, user.id, &token_hash, expires_at).await?;

        tracing::info!(user_id = %user.id, "Login successful");

        Ok((access_token, refresh_token))
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<(String, String), AppError> {
        let token_hash = hash_refresh_token(refresh_token);

        let db_refresh_token = match db_queries::find_refresh_token(&self.pool, &token_hash).await?
        {
            Some(db_refresh_token) => db_refresh_token,
            None => {
                return Err(AppError::Unauthorized(
                    "Invalid or expired refresh token, please log in again".to_string(),
                ));
            }
        };

        if chrono::Utc::now() > db_refresh_token.expires_at {
            db_queries::delete_refresh_token(&self.pool, &db_refresh_token.token_hash).await?;
            return Err(AppError::Unauthorized(
                "Invalid or expired refresh token, please log in again".to_string(),
            ));
        }

        let user_id = db_refresh_token.user_id;
        let user = match db_queries::find_user_by_id(&self.pool, user_id).await? {
            Some(user) => user,
            None => {
                tracing::error!(user_id = %user_id, "Refresh token references non-existent user");
                return Err(AppError::InternalError("User not found".to_string()));
            }
        };

        let access_token = create_token(user.id, user.role, &self.jwt_secret)?;
        let refresh_token = generate_refresh_token();
        let token_hash = hash_refresh_token(&refresh_token);
        let expires_at = chrono::Utc::now() + chrono::Duration::days(7);

        tracing::info!(user_id = %user_id, "Token refreshed");

        // delete old token
        db_queries::delete_refresh_token(&self.pool, &db_refresh_token.token_hash).await?;

        // create new one
        db_queries::create_refresh_token(&self.pool, user.id, &token_hash, expires_at).await?;

        Ok((access_token, refresh_token))
    }

    pub async fn logout(&self, refresh_token: &str) -> Result<(), AppError> {
        let token_hash = hash_refresh_token(refresh_token);

        let db_refresh_token = match db_queries::find_refresh_token(&self.pool, &token_hash).await?
        {
            Some(db_refresh_token) => db_refresh_token,
            None => {
                return Err(AppError::Unauthorized(
                    "Invalid or expired refresh token, please log in again".to_string(),
                ));
            }
        };

        db_queries::delete_refresh_token(&self.pool, &db_refresh_token.token_hash).await?;

        tracing::info!(user_id = %db_refresh_token.user_id, "Logout successful");

        Ok(())
    }
}
