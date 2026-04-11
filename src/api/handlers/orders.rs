use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::{
    api::{
        middleware::AuthUser,
        types::{
            AppState, PaginatedResponse,
            order::{GetOrdersParams, OrderResponse, PlaceOrderRequest, PlaceOrderResponse},
        },
    },
    constants::DEFAULT_PAGE_SIZE,
    error::AppError,
};

#[utoipa::path(
    post,
    path = "/api/v1/orders",
    tag = "Orders",
    request_body = PlaceOrderRequest,
    responses(
        (status = 200, description = "Order placed successfully", body = PlaceOrderResponse),
        (status = 400, description = "Invalid order request"),
        (status = 401, description = "Not authenticated"),
        (status = 422, description = "Insufficient balance or unsupported pair"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn place_order(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<PlaceOrderRequest>,
) -> Result<(StatusCode, Json<PlaceOrderResponse>), AppError> {
    // validate request body
    body.validate_request()?;

    let res = state
        .order_service
        .place_order(auth.0.user_id(), body)
        .await?;
    Ok((StatusCode::OK, Json(res)))
}

#[utoipa::path(
    get,
    path = "/api/v1/orders",
    tag = "Orders",
    params(
        ("status" = Option<String>, Query, description = "Filter by status: open, filled, cancelled, partially_filled"),
        ("pair" = Option<String>, Query, description = "Filter by trading pair e.g BTC/USDT"),
        ("page" = Option<u64>, Query, description = "Page number"),
        ("limit" = Option<u64>, Query, description = "Items per page"),
        ("order" = Option<String>, Query, description = "Sort order: asc or desc"),
    ),
    responses(
        (status = 200, description = "Orders fetched successfully", body = PaginatedResponse<OrderResponse>),
        (status = 401, description = "Not authenticated"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_orders(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(params): Query<GetOrdersParams>,
) -> Result<(StatusCode, Json<PaginatedResponse<OrderResponse>>), AppError> {
    let user_id = auth.0.user_id();

    let (orders, count) = state.order_service.get_orders(user_id, &params).await?;

    let orders = orders.into_iter().map(OrderResponse::from).collect();

    Ok((
        StatusCode::OK,
        Json(PaginatedResponse {
            data: orders,
            page: params.page.unwrap_or(1),
            limit: params.limit.unwrap_or(DEFAULT_PAGE_SIZE),
            total: count,
        }),
    ))
}

#[utoipa::path(
    delete,
    path = "/api/v1/orders/{order_id}",
    tag = "Orders",
    params(("order_id" = Uuid, Path, description = "Order ID")),
    responses(
        (status = 200, description = "Order cancelled successfully", body = OrderResponse),
        (status = 400, description = "Order cannot be cancelled"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Order does not belong to user"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn cancel_order(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(order_id): Path<Uuid>,
) -> Result<(StatusCode, Json<OrderResponse>), AppError> {
    let user_id = auth.0.user_id();
    let order_response = state.order_service.cancel_order(order_id, user_id).await?;

    Ok((StatusCode::OK, Json(order_response)))
}

#[utoipa::path(
    get,
    path = "/api/v1/orders/{order_id}",
    tag = "Orders",
    params(("order_id" = Uuid, Path, description = "Order ID")),
    responses(
        (status = 200, description = "Order fetched successfully", body = OrderResponse),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Order not found"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_order_by_id(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(order_id): Path<Uuid>,
) -> Result<(StatusCode, Json<OrderResponse>), AppError> {
    let (order, symbol) = state.order_service.get_order_by_id(order_id).await?;

    // check if order belongs to logged in user
    if order.user_id != auth.0.user_id() {
        return Err(AppError::NotFound("Order not found".to_string()));
    }
    Ok((StatusCode::OK, Json(OrderResponse::new(order, &symbol))))
}
