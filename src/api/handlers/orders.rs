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

pub async fn cancel_order(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(order_id): Path<Uuid>,
) -> Result<(StatusCode, Json<OrderResponse>), AppError> {
    let user_id = auth.0.user_id();
    let order_response = state.order_service.cancel_order(order_id, user_id).await?;

    Ok((StatusCode::OK, Json(order_response)))
}

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
