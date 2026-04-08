use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};

use crate::{
    api::{
        middleware::AdminUser,
        types::{
            AppState,
            trading_pairs::{AddTradingPairRequest, GetPairParams, TradingPairsResponse},
        },
    },
    error::AppError,
};

pub async fn get_active_trading_pairs(
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<Vec<TradingPairsResponse>>), AppError> {
    let res = state
        .trading_pair_service
        .get_active_trading_pairs()
        .await?;
    Ok((StatusCode::OK, Json(res)))
}

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

pub async fn get_trading_pair(
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
) -> Result<(StatusCode, Json<TradingPairsResponse>), AppError> {
    let res = state.trading_pair_service.get_trading_pair(&symbol).await?;
    Ok((StatusCode::OK, Json(res.into())))
}
