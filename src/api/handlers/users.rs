use std::sync::Arc;

use axum::http::StatusCode;
use axum::{Json, extract::State};

use crate::api::types::users::UpdateUserRequest;
use crate::{
    api::{
        middleware::AuthUser,
        types::{AppState, users::UserResponse},
    },
    error::AppError,
};

pub async fn get_user(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    let user_id = auth.0.user_id();
    let user = state.user_service.get_user(user_id).await?;

    Ok((StatusCode::OK, Json(user.into())))
}

pub async fn update_user(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    let user_id = auth.0.user_id();
    body.validate_request()?;
    let user = state
        .user_service
        .update_user(
            user_id,
            body.username.as_deref(),
            body.current_password.as_deref(),
            body.new_password.as_deref(),
        )
        .await?;
    Ok((StatusCode::OK, Json(user.into())))
}
