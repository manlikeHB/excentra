use std::{
    sync::{Arc, atomic::AtomicU64},
    time::Instant,
};

use crate::{
    services::{
        assets::AssetService, auth::AuthService, balances::BalanceService,
        orderbook::OrderBookService, orders::OrderService, ticker::TickerService,
        trades::TradeService, trading_pair::TradingPairService, users::UserService,
    },
    ws::messages::WsEvent,
};
use sqlx::PgPool;
use tokio::sync::broadcast;

pub struct AppState {
    pub pool: PgPool,
    pub order_service: OrderService,
    pub trading_pair_service: TradingPairService,
    pub trade_service: TradeService,
    pub asset_service: AssetService,
    pub order_book_service: OrderBookService,
    pub ws_sender: broadcast::Sender<WsEvent>,
    pub ticker_service: TickerService,
    pub ws_connections: Arc<AtomicU64>,
    pub started_at: Instant,
    pub auth_service: AuthService,
    pub user_service: UserService,
    pub base_url: String,
    pub balance_service: BalanceService,
}
