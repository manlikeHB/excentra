use crate::{
    services::{
        assets::AssetService, orderbook::OrderBookService, orders::OrderService,
        ticker::TickerService, trades::TradeService, trading_pair::TradingPairService,
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
    pub jwt_secret: String,
    pub ticker_service: TickerService,
}
