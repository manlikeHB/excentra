use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};

use crate::{
    api::{
        middleware::AuthUser,
        types::{
            AppState,
            order::{OrderResponse, PlaceOrderRequest, PlaceOrderResponse},
        },
    },
    db::queries::{self as db_queries},
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
) -> Result<(StatusCode, Json<Vec<OrderResponse>>), AppError> {
    let user_id = auth.0.user_id();

    let orders = db_queries::get_user_orders(&state.pool, user_id).await?;

    Ok((StatusCode::OK, Json(orders)))
}
