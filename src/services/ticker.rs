use rust_decimal::Decimal;
use sqlx::PgPool;
use tokio::sync::broadcast;
use tokio::time::{Duration, sleep};

use crate::{db::queries as db_queries, ws::messages::WsEvent};

pub struct TickerService {
    pool: PgPool,
    ws_sender: broadcast::Sender<WsEvent>,
}

pub struct Ticker {
    pub last_price: Decimal,
    pub high_24h: Decimal,
    pub low_24h: Decimal,
    pub volume_24h: Decimal,
    pub price_change_pct: Decimal,
}

impl TickerService {
    pub fn new(pool: PgPool, ws_sender: broadcast::Sender<WsEvent>) -> Self {
        TickerService { pool, ws_sender }
    }

    pub async fn run(&self) {
        loop {
            sleep(Duration::from_secs(5)).await;

            let pairs = match db_queries::get_active_trading_pairs(&self.pool).await {
                Ok(pairs) => pairs,
                Err(_) => continue, // skip this tick if DB fails
            };

            for pair in pairs {
                let trades =
                    match db_queries::get_trades_from_last_24_hours(&self.pool, pair.id).await {
                        Ok(t) if t.is_empty() => continue, // no trades, skip this pair
                        Ok(t) => t,
                        Err(_) => continue,
                    };

                // trades are ordered DESC so first is most recent
                let last_price = trades[0].price;

                let mut high_24h = Decimal::ZERO;
                let mut low_24h = Decimal::MAX;
                let mut volume_24h = Decimal::ZERO;

                for trade in &trades {
                    if trade.price > high_24h {
                        high_24h = trade.price;
                    }
                    if trade.price < low_24h {
                        low_24h = trade.price;
                    }
                    volume_24h += trade.quantity;
                }

                // get baseline price (last trade before the 24h window)
                // fall back to oldest trade in window if no baseline exists
                let baseline_price = match db_queries::get_baseline_trade(&self.pool, pair.id).await
                {
                    Ok(Some(t)) => t.price,
                    Ok(None) => trades.last().unwrap().price, // safe: never empty
                    Err(_) => trades.last().unwrap().price,   // safe: never empty
                };

                let price_change_pct =
                    (last_price - baseline_price) / baseline_price * Decimal::ONE_HUNDRED;

                let event = WsEvent::TickerUpdate {
                    symbol: pair.symbol,
                    last_price,
                    high_24h,
                    low_24h,
                    volume_24h,
                    price_change_pct,
                };

                let _ = self.ws_sender.send(event);
            }
        }
    }
}
