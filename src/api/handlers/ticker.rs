use std::{net::SocketAddr, sync::Arc};

use axum::{
    Json,
    extract::{ConnectInfo, Path, State},
    http::{HeaderMap, StatusCode},
};

use crate::{
    api::{
        middleware::rate_limit::policies,
        types::{AppState, ticker::TickerResponse},
    },
    error::AppError,
    types::asset_symbol::AssetSymbol,
    utils::ip_address::extract_ip,
};

#[utoipa::path(
    get,
    path = "/api/v1/ticker/{symbol}",
    tag = "Market Data",
    params(
        ("symbol" = String, Path, description = "Trading pair symbol e.g BTC/USDT"),
    ),
    responses(
        (status = 200, description = "Ticker fetched successfully", body = TickerResponse),
        (status = 400, description = "Invalid symbol"),
        (status = 404, description = "No trades found for this pair"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn get_ticker(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
    headers: HeaderMap,
) -> Result<(StatusCode, Json<TickerResponse>), AppError> {
    let ip = extract_ip(&headers, addr);
    state.rate_limiter.check(ip, &policies::GET_TICKER)?;

    let symbol = AssetSymbol::from_path(&symbol)?;
    let ticker = state.ticker_service.get_pair_ticker_stats(symbol).await?;

    Ok((StatusCode::OK, Json(ticker.into())))
}

#[utoipa::path(
    get,
    path = "/api/v1/ticker",
    tag = "Market Data",
    responses(
        (status = 200, description = "All tickers fetched successfully", body = Vec<TickerResponse>),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn get_all_tickers(
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<Vec<TickerResponse>>), AppError> {
    let tickers = state
        .ticker_service
        .get_all_tickers()
        .await?
        .into_iter()
        .map(|t| t.into())
        .collect();
    Ok((StatusCode::OK, Json(tickers)))
}
