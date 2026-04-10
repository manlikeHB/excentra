use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use reqwest::StatusCode;
use uuid::Uuid;

use crate::{
    api::{
        middleware::AdminUser,
        types::{
            AppState, PaginatedResponse,
            admin::{SuspendUserRequest, UpdateUserRoleRequest, UserSummary, UserSummaryParam},
            users::UserResponse,
        },
    },
    constants::DEFAULT_PAGE_SIZE,
    error::AppError,
};

pub async fn get_all_users_summary(
    _auth: AdminUser,
    State(state): State<Arc<AppState>>,
    Query(params): Query<UserSummaryParam>,
) -> Result<(StatusCode, Json<PaginatedResponse<UserSummary>>), AppError> {
    let (summaries, count) = state
        .admin_service
        .get_all_users_summary(params.page, params.limit, params.order)
        .await?;

    Ok((
        StatusCode::OK,
        Json(PaginatedResponse {
            data: summaries,
            page: params.page.unwrap_or(1),
            limit: params.limit.unwrap_or(DEFAULT_PAGE_SIZE),
            total: count,
        }),
    ))
}

pub async fn suspend_user(
    _auth: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Json(body): Json<SuspendUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    let user = state
        .admin_service
        .suspend_user(user_id, body.suspended)
        .await?;
    Ok((StatusCode::OK, Json(user.into())))
}

pub async fn update_role(
    _auth: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Json(body): Json<UpdateUserRoleRequest>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    let user = state.admin_service.update_role(user_id, body.role).await?;
    Ok((StatusCode::OK, Json(user.into())))
}
