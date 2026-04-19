use std::{net::SocketAddr, sync::Arc};

use axum::{
    Json,
    extract::{ConnectInfo, Path, Query, State},
    http::{HeaderMap, StatusCode},
};

use crate::{
    api::{
        middleware::rate_limit::policies,
        types::{
            AppState,
            orderbook::{OrderBookParams, OrderBookResponse},
        },
    },
    error::AppError,
    types::asset_symbol::AssetSymbol,
    utils::ip_address::extract_ip,
};

#[utoipa::path(
    get,
    path = "/api/v1/orderbook/{symbol}",
    tag = "Market Data",
    params(
        ("symbol" = String, Path, description = "Trading pair symbol e.g BTC/USDT"),
        ("levels" = Option<usize>, Query, description = "Number of price levels to return (default: 20)"),
    ),
    responses(
        (status = 200, description = "Order book snapshot", body = OrderBookResponse),
        (status = 400, description = "Invalid symbol"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn get_orderbook(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
    Query(params): Query<OrderBookParams>,
    headers: HeaderMap,
) -> Result<(StatusCode, Json<OrderBookResponse>), AppError> {
    let ip = extract_ip(&headers, addr);
    state.rate_limiter.check(ip, &policies::GET_ORDER_BOOK)?;

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
        bids: snapshot.bids().into_iter().map(|pl| pl.into()).collect(),
        asks: snapshot.asks().into_iter().map(|pl| pl.into()).collect(),
    };

    Ok((StatusCode::OK, Json(orderbook_res)))
}
