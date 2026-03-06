use crate::db::models::user::{User, UserRole};
use chrono::Utc;
use uuid::Uuid;
use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password should be at least 8 characters"))]
    pub password: String,
}

#[derive(serde::Serialize)]
pub struct RegisterResponse {
    pub id: Uuid,
    pub email: String,
    pub role: UserRole,
    pub created_at: chrono::DateTime<Utc>,
}

impl From<User> for RegisterResponse {
    fn from(value: User) -> Self {
        RegisterResponse {
            id: value.id,
            email: value.email,
            role: value.role,
            created_at: value.created_at,
        }
    }
}
