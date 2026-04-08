use crate::api::types::auth::{LoginRequest, LoginResponse};
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

    let token = state
        .auth_service
        .login(&body.email, &body.password)
        .await?;

    Ok((StatusCode::OK, Json(LoginResponse { token })))
}
