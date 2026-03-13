use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};

use crate::{
    api::types::{
        AppState,
        orderbook::{OrderBookParams, OrderBookResponse},
    },
    error::AppError,
    types::asset::AssetSymbol,
};

pub async fn get_orderbook(
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
    Query(params): Query<OrderBookParams>,
) -> Result<(StatusCode, Json<OrderBookResponse>), AppError> {
    let asset_symbol = AssetSymbol::from_path(&symbol)?;
    let levels = params.levels.unwrap_or(20);

    let pair_id = state
        .trading_pair_service
        .get_pair_id(&asset_symbol)
        .await?;

    let snapshot = state
        .order_book_service
        .get_orderbook(pair_id, levels)
        .await?;

    let orderbook_res = OrderBookResponse {
        symbol: asset_symbol.as_str().to_string(),
        bids: snapshot.bids(),
        asks: snapshot.asks(),
    };

    Ok((StatusCode::OK, Json(orderbook_res)))
}
