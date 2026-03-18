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
                Err(_) => continue,
            };

            for pair in pairs {
                let stat = match db_queries::get_trade_stats(&self.pool, pair.id).await {
                    Ok(s) => s,
                    Err(_) => continue,
                };

                // if any of these are None, there are no trades — skip
                let (Some(high_24h), Some(low_24h), Some(volume_24h), Some(last_price)) = (
                    stat.high_24h,
                    stat.low_24h,
                    stat.volume_24h,
                    stat.last_price,
                ) else {
                    continue;
                };

                // use baseline if available, fall back to oldest trade in window
                let Some(baseline_price) = stat.baseline_price.or(stat.oldest_price) else {
                    continue;
                };

                let price_change_pct =
                    (last_price - baseline_price) / baseline_price * Decimal::ONE_HUNDRED;

                let _ = self.ws_sender.send(WsEvent::TickerUpdate {
                    symbol: pair.symbol,
                    last_price,
                    high_24h,
                    low_24h,
                    volume_24h,
                    price_change_pct,
                });
            }
        }
    }
}
