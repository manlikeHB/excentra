use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};

use crate::{
    api::types::{
        AppState,
        password_reset::{ForgotPasswordRequest, ResetPasswordRequest},
    },
    error::AppError,
};

#[utoipa::path(
    post,
    path = "/api/v1/auth/forgot-password",
    tag = "Auth",
    request_body = ForgotPasswordRequest,
    responses(
        (status = 204, description = "Reset email sent if account exists"),
        (status = 400, description = "Invalid request body"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn request_password_reset(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ForgotPasswordRequest>,
) -> Result<StatusCode, AppError> {
    state
        .password_reset_service
        .request_reset(&body.email)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/reset-password",
    tag = "Auth",
    request_body = ResetPasswordRequest,
    responses(
        (status = 204, description = "Password reset successfully"),
        (status = 400, description = "Invalid or expired reset token"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn reset_password(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ResetPasswordRequest>,
) -> Result<StatusCode, AppError> {
    state
        .password_reset_service
        .reset_password(&body.token, &body.new_password)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
