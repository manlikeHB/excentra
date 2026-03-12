use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::{
    api::types::{AppState, trades::TradeResponse},
    engine::models::asset::AssetSymbol,
    error::AppError,
};

pub async fn get_recent_trades(
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
) -> Result<(StatusCode, Json<Vec<TradeResponse>>), AppError> {
    let asset_symbol = AssetSymbol::from_path(&symbol)?;

    // TODO: limit
    let trades = state.trade_service.get_trades(&asset_symbol, 100).await?;

    let mut t_vec = Vec::new();
    for t in trades {
        t_vec.push(TradeResponse::new(t, asset_symbol.as_str().to_string()));
    }

    Ok((StatusCode::OK, Json(t_vec)))
}
