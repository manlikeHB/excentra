use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::{
    api::types::AppState, error::AppError, services::ticker::Ticker,
    types::asset_symbol::AssetSymbol,
};

pub async fn get_ticker(
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
) -> Result<(StatusCode, Json<Ticker>), AppError> {
    let symbol = AssetSymbol::from_path(&symbol)?;
    let ticker = state.ticker_service.get_pair_ticker_stats(symbol).await?;

    Ok((StatusCode::OK, Json(ticker)))
}

pub async fn get_all_tickers(
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<Vec<Ticker>>), AppError> {
    let tickers = state.ticker_service.get_all_tickers().await?;
    Ok((StatusCode::OK, Json(tickers)))
}
