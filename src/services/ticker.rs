use rust_decimal::Decimal;
use sqlx::PgPool;
use tokio::sync::broadcast;
use tokio::time::{Duration, sleep};

use crate::db::models::trade::TradeStat;
use crate::error::AppError;
use crate::types::asset_symbol::AssetSymbol;
use crate::{db::queries as db_queries, ws::messages::WsEvent};

pub struct TickerService {
    pool: PgPool,
    ws_sender: broadcast::Sender<WsEvent>,
}

#[derive(Debug, serde::Serialize)]
pub struct Ticker {
    pub last_price: Decimal,
    pub high_24h: Decimal,
    pub low_24h: Decimal,
    pub volume_24h: Decimal,
    pub price_change_pct: Decimal,
}

impl Ticker {
    pub fn new(
        last_price: Decimal,
        high_24h: Decimal,
        low_24h: Decimal,
        volume_24h: Decimal,
        price_change_pct: Decimal,
    ) -> Self {
        Ticker {
            last_price,
            high_24h,
            low_24h,
            volume_24h,
            price_change_pct,
        }
    }
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

                // if there is no ticker — skip
                let ticker = match Self::get_ticker(stat) {
                    Some(t) => t,
                    None => continue,
                };

                let _ = self.ws_sender.send(WsEvent::TickerUpdate {
                    symbol: pair.symbol,
                    last_price: ticker.last_price,
                    high_24h: ticker.last_price,
                    low_24h: ticker.low_24h,
                    volume_24h: ticker.volume_24h,
                    price_change_pct: ticker.price_change_pct,
                });
            }
        }
    }

    pub async fn get_pair_ticker_stats(&self, symbol: AssetSymbol) -> Result<Ticker, AppError> {
        let tp = match db_queries::find_by_symbol(&self.pool, symbol.as_str()).await? {
            Some(tp) => tp,
            None => return Err(AppError::BadRequest("Invalid pair symbol".to_string())),
        };

        let ts = db_queries::get_trade_stats(&self.pool, tp.id).await?;

        let ticker = match Self::get_ticker(ts) {
            Some(t) => t,
            None => {
                return Err(AppError::NotFound(
                    "No trades found for this pair".to_string(),
                ));
            }
        };

        Ok(ticker)
    }

    pub fn get_ticker(stat: TradeStat) -> Option<Ticker> {
        // if any of these are None, there are no trades — return None
        let (Some(high_24h), Some(low_24h), Some(volume_24h), Some(last_price)) = (
            stat.high_24h,
            stat.low_24h,
            stat.volume_24h,
            stat.last_price,
        ) else {
            return None;
        };

        // use baseline if available, fall back to oldest trade in window
        let Some(baseline_price) = stat.baseline_price.or(stat.oldest_price) else {
            return None;
        };

        let price_change_pct =
            (last_price - baseline_price) / baseline_price * Decimal::ONE_HUNDRED;

        Some(Ticker::new(
            last_price,
            high_24h,
            low_24h,
            volume_24h,
            price_change_pct,
        ))
    }
}
