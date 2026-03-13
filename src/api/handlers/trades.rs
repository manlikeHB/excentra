use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};

use crate::{
    api::types::{
        AppState,
        trades::{TradeParams, TradeResponse},
    },
    error::AppError,
    types::asset::AssetSymbol,
};

pub async fn get_recent_trades(
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
    Query(params): Query<TradeParams>,
) -> Result<(StatusCode, Json<Vec<TradeResponse>>), AppError> {
    let asset_symbol = AssetSymbol::from_path(&symbol)?;
    let limit = params.limit.unwrap_or(20);

    let trades = state.trade_service.get_trades(&asset_symbol, limit).await?;

    let trades = trades
        .into_iter()
        .map(|t| TradeResponse::new(t, asset_symbol.as_str().to_string()))
        .collect();

    Ok((StatusCode::OK, Json(trades)))
}
