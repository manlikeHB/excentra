use crate::api::types::auth::{LoginRequest, LoginResponse, RefreshTokenRequest};
use crate::{
    api::types::{
        AppState,
        auth::{RegisterRequest, RegisterResponse},
    },
    error::AppError,
};
use axum::{Json, extract::State, http::StatusCode};
use std::sync::Arc;
use validator::Validate;

pub async fn register_user(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<RegisterResponse>), AppError> {
    // validate email
    body.validate()?;
    let user = state
        .auth_service
        .register_user(&body.email, &body.password)
        .await?;

    Ok((StatusCode::CREATED, Json(user.into())))
}

pub async fn login_user(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoginRequest>,
) -> Result<(StatusCode, Json<LoginResponse>), AppError> {
    body.validate()?;

    let (access_token, refresh_token) = state
        .auth_service
        .login(&body.email, &body.password)
        .await?;

    // TODO: set refresh_token as httpOnly cookie
    Ok((
        StatusCode::OK,
        Json(LoginResponse {
            access_token,
            refresh_token,
        }),
    ))
}

pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RefreshTokenRequest>,
) -> Result<(StatusCode, Json<LoginResponse>), AppError> {
    let (access_token, refresh_token) = state
        .auth_service
        .refresh_token(&body.refresh_token)
        .await?;

    // TODO: set refresh_token as httpOnly cookie
    Ok((
        StatusCode::OK,
        Json(LoginResponse {
            access_token,
            refresh_token,
        }),
    ))
}

pub async fn logout(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RefreshTokenRequest>,
) -> Result<StatusCode, AppError> {
    state.auth_service.logout(&body.refresh_token).await?;
    Ok(StatusCode::OK)
}
