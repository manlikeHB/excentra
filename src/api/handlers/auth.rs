use crate::api::types::auth::{LoginRequest, LoginResponse};
use crate::auth::{create_token, hash_password, verify_password};
use crate::db::queries::users::{create_user, find_by_email};
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

    let password_hash = hash_password(&body.password)?;

    let new_user = create_user(&state.pool, &body.email, &password_hash).await?;

    Ok((StatusCode::CREATED, Json(new_user.into())))
}

pub async fn login_user(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoginRequest>,
) -> Result<(StatusCode, Json<LoginResponse>), AppError> {
    body.validate()?;

    let user = match find_by_email(&state.pool, &body.email).await? {
        Some(user) => user,
        None => {
            return Err(AppError::Unauthorized(
                "Invalid email or password".to_string(),
            ));
        }
    };

    match verify_password(&body.password, &user.password_hash)? {
        true => (),
        false => {
            return Err(AppError::Unauthorized(
                "Invalid email or password".to_string(),
            ));
        }
    };

    let token = create_token(user.id, user.role, &state.jwt_secret)?;

    Ok((StatusCode::OK, Json(LoginResponse { token })))
}
