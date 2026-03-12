use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};

use crate::{
    api::{
        middleware::AuthUser,
        types::{
            AppState,
            trading_pairs::{AddTradingPairRequest, TradingPairsResponse},
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

// TODO: restrict to admin
pub async fn get_all_trading_pairs(
    _auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<Vec<TradingPairsResponse>>), AppError> {
    let res = state.trading_pair_service.get_all_trading_pairs().await?;
    Ok((StatusCode::OK, Json(res)))
}

// TODO: restrict to admin
pub async fn add_trading_pair(
    _auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<AddTradingPairRequest>,
) -> Result<(StatusCode, Json<TradingPairsResponse>), AppError> {
    // TODO: normalize request body
    let res = state
        .trading_pair_service
        .add_trading_pair(&body.base_asset, &body.quote_asset)
        .await?;
    Ok((StatusCode::OK, Json(res.into())))
}
