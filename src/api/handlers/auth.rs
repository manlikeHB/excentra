use crate::auth::hash_password;
use crate::db::queries::users::create_user;
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
