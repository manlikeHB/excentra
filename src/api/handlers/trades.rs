use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};

use crate::{
    api::{
        middleware::AuthUser,
        types::{
            AppState, PaginatedResponse,
            trades::{TradeParams, TradeResponse, UserTradeResponse},
        },
    },
    constants::DEFAULT_PAGE_SIZE,
    error::AppError,
    types::asset_symbol::AssetSymbol,
};

pub async fn get_recent_trades_for_a_pair(
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
    Query(params): Query<TradeParams>,
) -> Result<(StatusCode, Json<Vec<TradeResponse>>), AppError> {
    let asset_symbol = AssetSymbol::from_path(&symbol)?;
    let limit = params.limit.unwrap_or(DEFAULT_PAGE_SIZE);

    let trades = state.trade_service.get_trades(&asset_symbol, limit).await?;

    let trades = trades
        .into_iter()
        .map(|t| TradeResponse::new(t, asset_symbol.as_str().to_string()))
        .collect();

    Ok((StatusCode::OK, Json(trades)))
}

pub async fn get_trade_history(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(params): Query<TradeParams>,
) -> Result<(StatusCode, Json<PaginatedResponse<UserTradeResponse>>), AppError> {
    let user_id = auth.0.user_id();

    let (trade, count) = state
        .trade_service
        .get_trade_history(user_id, params.pair.as_deref(), params.page, params.limit)
        .await?;

    Ok((
        StatusCode::OK,
        Json(PaginatedResponse {
            data: trade.into_iter().map(|t| t.into()).collect(),
            page: params.page.unwrap_or(1),
            limit: params.limit.unwrap_or(DEFAULT_PAGE_SIZE),
            total: count,
        }),
    ))
}
