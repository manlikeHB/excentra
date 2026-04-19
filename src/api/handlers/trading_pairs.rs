use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};

use crate::{
    api::{
        middleware::auth::AdminUser,
        types::{
            AppState,
            trading_pairs::{AddTradingPairRequest, GetPairParams, TradingPairsResponse},
        },
    },
    error::AppError,
};

#[utoipa::path(
    get,
    path = "/api/v1/pairs/active",
    tag = "Market Data",
    responses(
        (status = 200, description = "Active trading pairs fetched", body = Vec<TradingPairsResponse>),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn get_active_trading_pairs(
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<Vec<TradingPairsResponse>>), AppError> {
    let res = state
        .trading_pair_service
        .get_active_trading_pairs()
        .await?;
    Ok((StatusCode::OK, Json(res)))
}

#[utoipa::path(
    get,
    path = "/api/v1/pairs",
    tag = "Admin",
    params(
        ("active" = Option<bool>, Query, description = "Filter by active status"),
    ),
    responses(
        (status = 200, description = "All trading pairs fetched", body = Vec<TradingPairsResponse>),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Admin access required"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_all_trading_pairs(
    _auth: AdminUser,
    State(state): State<Arc<AppState>>,
    Query(params): Query<GetPairParams>,
) -> Result<(StatusCode, Json<Vec<TradingPairsResponse>>), AppError> {
    let res = state
        .trading_pair_service
        .get_all_trading_pairs(params.active)
        .await?;
    Ok((StatusCode::OK, Json(res)))
}

#[utoipa::path(
    post,
    path = "/api/v1/pairs",
    tag = "Admin",
    request_body = AddTradingPairRequest,
    responses(
        (status = 201, description = "Trading pair created", body = TradingPairsResponse),
        (status = 400, description = "Unsupported asset"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Admin access required"),
        (status = 409, description = "Trading pair already exists"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn add_trading_pair(
    _auth: AdminUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<AddTradingPairRequest>,
) -> Result<(StatusCode, Json<TradingPairsResponse>), AppError> {
    let res = state
        .trading_pair_service
        .add_trading_pair(
            &body.base_asset.to_uppercase(),
            &body.quote_asset.to_uppercase(),
        )
        .await?;
    Ok((StatusCode::CREATED, Json(res.into())))
}

#[utoipa::path(
    get,
    path = "/api/v1/pairs/{symbol}",
    tag = "Market Data",
    params(
        ("symbol" = String, Path, description = "Trading pair symbol e.g BTC/USDT"),
    ),
    responses(
        (status = 200, description = "Trading pair fetched", body = TradingPairsResponse),
        (status = 404, description = "Trading pair not found"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn get_trading_pair(
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
) -> Result<(StatusCode, Json<TradingPairsResponse>), AppError> {
    let res = state.trading_pair_service.get_trading_pair(&symbol).await?;
    Ok((StatusCode::OK, Json(res.into())))
}
