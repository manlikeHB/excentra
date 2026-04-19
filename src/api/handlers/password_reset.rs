use std::{net::SocketAddr, sync::Arc};

use axum::{
    Json,
    extract::{ConnectInfo, State},
    http::{HeaderMap, StatusCode},
};

use crate::{
    api::{
        middleware::rate_limit::policies,
        types::{
            AppState,
            password_reset::{ForgotPasswordRequest, ResetPasswordRequest},
        },
    },
    error::AppError,
    utils::ip_address::extract_ip,
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
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(body): Json<ForgotPasswordRequest>,
) -> Result<StatusCode, AppError> {
    let ip = extract_ip(&headers, addr);
    state.rate_limiter.check(ip, &policies::FORGOT_PASSWORD)?;

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
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(body): Json<ResetPasswordRequest>,
) -> Result<StatusCode, AppError> {
    let ip = extract_ip(&headers, addr);
    state.rate_limiter.check(ip, &policies::RESET_PASSWORD)?;

    state
        .password_reset_service
        .reset_password(&body.token, &body.new_password)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
