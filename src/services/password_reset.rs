use sqlx::PgPool;

use crate::services::auth::utils::hash_password;
use crate::utils::random_token;
use crate::{db::queries as db_queries, error::AppError};
use lettre::{Message, SmtpTransport, Transport};

pub struct PasswordResetService {
    pool: PgPool,
    smtp_config: SMTPConfig,
}

pub struct SMTPConfig {
    host: String,
    port: u16,
    from_address: String,
    frontend_url: String,
}

impl SMTPConfig {
    fn new(host: &str, port: u16, from_address: &str, frontend_url: &str) -> Self {
        SMTPConfig {
            host: host.to_string(),
            port,
            from_address: from_address.to_string(),
            frontend_url: frontend_url.to_string(),
        }
    }
}

impl PasswordResetService {
    pub fn new(
        pool: PgPool,
        host: &str,
        port: u16,
        from_address: &str,
        frontend_url: &str,
    ) -> Self {
        let smtp_config = SMTPConfig::new(host, port, from_address, frontend_url);
        PasswordResetService { pool, smtp_config }
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

        // store token hash
        db_queries::create_reset_token(&self.pool, user.id, &token_hash, expires_at).await?;

        let message = Message::builder()
            .from(self.smtp_config.from_address.parse().map_err(|e| {
                tracing::error!(error = %e, "Invalid sender address in SMTP config");
                AppError::InternalError("Invalid sender address configuration".to_string())
            })?)
            .to(email.parse().map_err(|e| {
                tracing::warn!(error = %e, "Invalid email address provided for password reset");
                AppError::BadRequest("Invalid email address".to_string())
            })?)
            .subject("Password Reset")
            .body(format!(
                "{}/reset-password?token={}",
                &self.smtp_config.frontend_url, token
            ))?;

        // TODO: replace with TLS transport + auth credentials for production
        let mailer = SmtpTransport::builder_dangerous(&self.smtp_config.host)
            .port(self.smtp_config.port)
            .build();

        mailer
            .send(&message)
            .map_err(|_| AppError::InternalError("Failed to send email".to_string()))?;

        tracing::info!(user_id = %user.id, "Password reset email sent");

        Ok(())
    }

    pub async fn reset_password(&self, token: &str, new_password: &str) -> Result<(), AppError> {
        let token_hash = random_token::hash_token(token);

        let reset_token = match db_queries::get_valid_reset_token(&self.pool, &token_hash).await? {
            Some(reset_token) => reset_token,
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
