use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::{
    api::{
        middleware::AdminUser,
        types::{
            AppState, PaginatedResponse,
            admin::{
                AdminStats, SuspendUserRequest, UpdateUserRoleRequest, UserSummary,
                UserSummaryParam,
            },
            users::UserResponse,
        },
    },
    constants::DEFAULT_PAGE_SIZE,
    error::AppError,
};

#[utoipa::path(
    get,
    path = "/api/v1/admin/users",
    tag = "Admin",
    params(
        ("page" = Option<u64>, Query, description = "Page number"),
        ("limit" = Option<u64>, Query, description = "Items per page"),
        ("order" = Option<String>, Query, description = "Sort order: asc or desc"),
    ),
    responses(
        (status = 200, description = "Users fetched successfully", body = PaginatedResponse<UserSummary>),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Admin access required"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
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

#[utoipa::path(
    patch,
    path = "/api/v1/admin/users/{user_id}/suspend",
    tag = "Admin",
    params(("user_id" = Uuid, Path, description = "User ID")),
    request_body = SuspendUserRequest,
    responses(
        (status = 200, description = "User suspension updated", body = UserResponse),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Admin access required"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
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

#[utoipa::path(
    patch,
    path = "/api/v1/admin/users/{user_id}/role",
    tag = "Admin",
    params(("user_id" = Uuid, Path, description = "User ID")),
    request_body = UpdateUserRoleRequest,
    responses(
        (status = 200, description = "User role updated", body = UserResponse),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Admin access required"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_role(
    _auth: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Json(body): Json<UpdateUserRoleRequest>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    let user = state.admin_service.update_role(user_id, body.role).await?;
    Ok((StatusCode::OK, Json(user.into())))
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/stats",
    tag = "Admin",
    responses(
        (status = 200, description = "System stats fetched", body = AdminStats),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Admin access required"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_admin_stat(
    _auth: AdminUser,
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<AdminStats>), AppError> {
    let stats = state
        .admin_service
        .get_stats(
            state
                .ws_connections
                .load(std::sync::atomic::Ordering::Relaxed),
            state.order_service.orders_processed(),
            state.started_at.elapsed().as_secs(),
        )
        .await?;
    Ok((StatusCode::OK, Json(stats)))
}
