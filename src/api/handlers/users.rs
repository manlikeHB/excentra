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

#[utoipa::path(
    get,
    path = "/api/v1/users/me",
    tag = "Users",
    responses(
        (status = 200, description = "User profile fetched", body = UserResponse),
        (status = 401, description = "Not authenticated"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_user(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    let user_id = auth.0.user_id();
    let user = state.user_service.get_user(user_id).await?;

    Ok((StatusCode::OK, Json(user.into())))
}

#[utoipa::path(
    patch,
    path = "/api/v1/users/me",
    tag = "Users",
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User profile updated", body = UserResponse),
        (status = 400, description = "Invalid request body"),
        (status = 401, description = "Not authenticated or incorrect current password"),
        (status = 409, description = "Username already taken"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
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
