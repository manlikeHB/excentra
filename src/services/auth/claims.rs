use uuid::Uuid;

use crate::db::models::user::UserRole;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Claims {
    user_id: Uuid,
    role: UserRole,
    exp: i64,
}

impl Claims {
    pub fn new(user_id: Uuid, role: UserRole, exp: i64) -> Self {
        Claims { user_id, role, exp }
    }

    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    pub fn role(&self) -> UserRole {
        self.role
    }
    pub fn exp(&self) -> i64 {
        self.exp
    }
}
