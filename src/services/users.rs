use sqlx::PgPool;
use uuid::Uuid;

use crate::db::models::user::User;
use crate::db::queries as db_queries;
use crate::error::AppError;
use crate::services::auth::utils::{hash_password, verify_password};

pub struct UserService {
    pool: PgPool,
}

impl UserService {
    pub fn new(pool: PgPool) -> Self {
        UserService { pool }
    }

    pub async fn get_user(&self, user_id: Uuid) -> Result<User, AppError> {
        let user = match db_queries::find_by_id(&self.pool, user_id).await? {
            Some(u) => u,
            None => {
                return Err(AppError::Unauthorized(
                    "Session invalid, please log in again".to_string(),
                ));
            }
        };

        Ok(user)
    }

    pub async fn update_user(
        &self,
        user_id: Uuid,
        username: Option<&str>,
        current_password: Option<&str>,
        new_password: Option<&str>,
    ) -> Result<User, AppError> {
        if let Some(p) = current_password {
            let user = match db_queries::find_by_id(&self.pool, user_id).await? {
                Some(u) => u,
                None => {
                    return Err(AppError::Unauthorized(
                        "Session invalid, please log in again".to_string(),
                    ));
                }
            };

            match verify_password(p, &user.password_hash)? {
                true => (),
                false => {
                    return Err(AppError::Unauthorized(
                        "Current password is incorrect".to_string(),
                    ));
                }
            };
        }

        match (username, new_password) {
            (Some(n), Some(p)) => {
                let hash = hash_password(p)?;
                match db_queries::update_username_and_password(&self.pool, user_id, n, &hash)
                    .await?
                {
                    Some(user) => Ok(user),
                    None => {
                        return Err(AppError::InternalError("Failed to update user".to_string()));
                    }
                }
            }
            (Some(n), None) => match db_queries::update_username(&self.pool, user_id, n).await? {
                Some(user) => Ok(user),
                None => return Err(AppError::InternalError("Failed to update user".to_string())),
            },
            (None, Some(p)) => {
                let hash = hash_password(p)?;
                match db_queries::update_password(&self.pool, user_id, &hash).await? {
                    Some(user) => Ok(user),
                    None => {
                        return Err(AppError::InternalError("Failed to update user".to_string()));
                    }
                }
            }
            (None, None) => {
                return Err(AppError::BadRequest("No fields to update".to_string()));
            }
        }
    }
}
