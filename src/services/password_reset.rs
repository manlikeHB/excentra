use sqlx::PgPool;

use crate::services::auth::utils::hash_password;
use crate::utils::random_token;
use crate::{db::queries as db_queries, error::AppError};
use resend_rs::{Resend, types::CreateEmailBaseOptions};

pub struct PasswordResetService {
    pool: PgPool,
    resend_api_key: Option<String>,
    from_address: String,
    frontend_url: String,
}

impl PasswordResetService {
    pub fn new(
        pool: PgPool,
        resend_api_key: Option<String>,
        from_address: &str,
        frontend_url: &str,
    ) -> Self {
        PasswordResetService {
            pool,
            resend_api_key,
            from_address: from_address.to_string(),
            frontend_url: frontend_url.to_string(),
        }
    }

    pub async fn request_reset(&self, email: &str) -> Result<(), AppError> {
        let user = match db_queries::find_user_by_email(&self.pool, email).await? {
            Some(u) => u,
            None => {
                tracing::warn!("Password reset attempted for unregistered email");
                return Ok(());
            }
        };

        let token = random_token::generate_token();
        let token_hash = random_token::hash_token(&token);
        let expires_at = chrono::Utc::now() + chrono::Duration::minutes(15);

        db_queries::create_reset_token(&self.pool, user.id, &token_hash, expires_at).await?;

        let reset_url = format!("{}/reset-password?token={}", self.frontend_url, token);

        match &self.resend_api_key {
            Some(api_key) => {
                self.send_via_resend(api_key, email, &reset_url).await?;
            }
            None => {
                // Dev mode — log the reset URL instead of sending email
                tracing::warn!(
                    user_id = %user.id,
                    reset_url = %reset_url,
                    "DEV MODE: Password reset email not sent. Use this URL."
                );
            }
        }

        tracing::info!(user_id = %user.id, "Password reset flow completed");
        Ok(())
    }

    async fn send_via_resend(
        &self,
        api_key: &str,
        to: &str,
        reset_url: &str,
    ) -> Result<(), AppError> {
        let resend = Resend::new(api_key);

        let email =
            CreateEmailBaseOptions::new(&self.from_address, [to], "Reset your Excentra password")
                .with_html(&format!(
                    "<p>Click the link below to reset your password. It expires in 15 minutes.</p>\
         <p><a href=\"{0}\">{0}</a></p>",
                    reset_url
                ));

        resend.emails.send(email).await.map_err(|e| {
            tracing::error!(error = %e, "Resend SDK error");
            AppError::InternalError("Failed to send email".to_string())
        })?;

        Ok(())
    }

    pub async fn reset_password(&self, token: &str, new_password: &str) -> Result<(), AppError> {
        let token_hash = random_token::hash_token(token);

        let reset_token = match db_queries::get_valid_reset_token(&self.pool, &token_hash).await? {
            Some(t) => t,
            None => {
                tracing::warn!("Invalid or expired reset token");
                return Err(AppError::BadRequest(
                    "Invalid or expired reset token".to_string(),
                ));
            }
        };

        let password_hash = hash_password(new_password)?;
        db_queries::update_password(&self.pool, reset_token.user_id, &password_hash).await?;
        db_queries::mark_reset_token_used(&self.pool, reset_token.id).await?;

        tracing::info!(user_id = %reset_token.user_id, "Password reset successful");
        Ok(())
    }
}
