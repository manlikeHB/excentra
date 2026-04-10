use chrono::Utc;
use uuid::Uuid;

use crate::{
    db::models::user::{User, UserRole},
    error::AppError,
};
use validator::Validate;

#[derive(Debug, serde::Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: Option<String>,
    pub email: String,
    pub role: UserRole,
    pub is_suspended: bool,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        UserResponse {
            id: u.id,
            username: u.username,
            email: u.email,
            role: u.role,
            is_suspended: u.is_suspended,
            created_at: u.created_at,
            updated_at: u.updated_at,
        }
    }
}

#[derive(Debug, serde::Deserialize, Validate)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub current_password: Option<String>,
    pub new_password: Option<String>,
}

pub enum UpdateUserRequestValidationError {
    MissingCurrentOrNewPassword,
    InvalidRequestBody,
    EmptyField,
    InvalidLength,
    InvalidUsername,
}

impl UpdateUserRequest {
    pub fn validate_request(&self) -> Result<(), UpdateUserRequestValidationError> {
        if self.current_password.is_none() && self.new_password.is_none() && self.username.is_none()
        {
            return Err(UpdateUserRequestValidationError::InvalidRequestBody);
        }

        match (
            self.current_password.as_deref(),
            self.new_password.as_deref(),
        ) {
            (Some(_), None) | (None, Some(_)) => {
                return Err(UpdateUserRequestValidationError::MissingCurrentOrNewPassword);
            }
            (Some(c), Some(n)) => {
                let c = c.trim();
                let n = n.trim();
                if c.is_empty() || n.is_empty() {
                    return Err(UpdateUserRequestValidationError::EmptyField);
                };

                if (c.len() < 8) || (n.len() < 8) {
                    return Err(UpdateUserRequestValidationError::InvalidLength);
                };
            }
            _ => {}
        };

        if let Some(username) = self.username.as_deref() {
            let username = username.trim();
            if username.is_empty() {
                return Err(UpdateUserRequestValidationError::EmptyField);
            };

            if !username.chars().all(|c| c.is_alphanumeric() || c == '_')
                || username.contains(" ")
                || username.len() < 3
            {
                return Err(UpdateUserRequestValidationError::InvalidUsername);
            }
        };

        Ok(())
    }
}

impl From<UpdateUserRequestValidationError> for AppError {
    fn from(error: UpdateUserRequestValidationError) -> Self {
        match error {
            UpdateUserRequestValidationError::InvalidRequestBody => {
                AppError::BadRequest("Request body cannot be empty".to_string())
            }
            UpdateUserRequestValidationError::MissingCurrentOrNewPassword => {
                AppError::BadRequest("Provide current and new password".to_string())
            }
            UpdateUserRequestValidationError::EmptyField => {
                AppError::BadRequest("Fields cannot be empty".to_string())
            }
            UpdateUserRequestValidationError::InvalidLength => {
                AppError::BadRequest("password should be at least 8 characters".to_string())
            }
            UpdateUserRequestValidationError::InvalidUsername => {
                AppError::BadRequest("Username should be min of 3 characters, no spaces, alphanumeric + underscores only".to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_request(
        username: Option<&str>,
        current_password: Option<&str>,
        new_password: Option<&str>,
    ) -> UpdateUserRequest {
        UpdateUserRequest {
            username: username.map(|s| s.to_string()),
            current_password: current_password.map(|s| s.to_string()),
            new_password: new_password.map(|s| s.to_string()),
        }
    }

    // empty request
    #[test]
    fn test_empty_request_fails() {
        let req = make_request(None, None, None);
        assert!(matches!(
            req.validate_request(),
            Err(UpdateUserRequestValidationError::InvalidRequestBody)
        ));
    }

    // password tests
    #[test]
    fn test_new_password_without_current_fails() {
        let req = make_request(None, None, Some("newpass123"));
        assert!(matches!(
            req.validate_request(),
            Err(UpdateUserRequestValidationError::MissingCurrentOrNewPassword)
        ));
    }

    #[test]
    fn test_current_password_without_new_fails() {
        let req = make_request(None, Some("currentpass123"), None);
        assert!(matches!(
            req.validate_request(),
            Err(UpdateUserRequestValidationError::MissingCurrentOrNewPassword)
        ));
    }

    #[test]
    fn test_empty_current_password_fails() {
        let req = make_request(None, Some("  "), Some("newpass123"));
        assert!(matches!(
            req.validate_request(),
            Err(UpdateUserRequestValidationError::EmptyField)
        ));
    }

    #[test]
    fn test_empty_new_password_fails() {
        let req = make_request(None, Some("currentpass123"), Some("  "));
        assert!(matches!(
            req.validate_request(),
            Err(UpdateUserRequestValidationError::EmptyField)
        ));
    }

    #[test]
    fn test_short_current_password_fails() {
        let req = make_request(None, Some("short"), Some("newpass123"));
        assert!(matches!(
            req.validate_request(),
            Err(UpdateUserRequestValidationError::InvalidLength)
        ));
    }

    #[test]
    fn test_short_new_password_fails() {
        let req = make_request(None, Some("currentpass123"), Some("short"));
        assert!(matches!(
            req.validate_request(),
            Err(UpdateUserRequestValidationError::InvalidLength)
        ));
    }

    #[test]
    fn test_valid_password_change() {
        let req = make_request(None, Some("currentpass123"), Some("newpass123"));
        assert!(req.validate_request().is_ok());
    }

    // username tests
    #[test]
    fn test_empty_username_fails() {
        let req = make_request(Some("  "), None, None);
        assert!(matches!(
            req.validate_request(),
            Err(UpdateUserRequestValidationError::EmptyField)
        ));
    }

    #[test]
    fn test_short_username_fails() {
        let req = make_request(Some("ab"), None, None);
        assert!(matches!(
            req.validate_request(),
            Err(UpdateUserRequestValidationError::InvalidUsername)
        ));
    }

    #[test]
    fn test_username_with_spaces_fails() {
        let req = make_request(Some("mik e"), None, None);
        assert!(matches!(
            req.validate_request(),
            Err(UpdateUserRequestValidationError::InvalidUsername)
        ));
    }

    #[test]
    fn test_username_with_special_chars_fails() {
        let req = make_request(Some("c@ne"), None, None);
        assert!(matches!(
            req.validate_request(),
            Err(UpdateUserRequestValidationError::InvalidUsername)
        ));
    }

    #[test]
    fn test_valid_username() {
        let req = make_request(Some("cole_123"), None, None);
        assert!(req.validate_request().is_ok());
    }

    // combined tests
    #[test]
    fn test_valid_username_and_password_change() {
        let req = make_request(Some("sam_123"), Some("currentpass123"), Some("newpass123"));
        assert!(req.validate_request().is_ok());
    }

    #[test]
    fn test_invalid_username_with_valid_password_change_fails() {
        let req = make_request(Some("s@mmuel"), Some("currentpass123"), Some("newpass123"));
        assert!(matches!(
            req.validate_request(),
            Err(UpdateUserRequestValidationError::InvalidUsername)
        ));
    }
}
