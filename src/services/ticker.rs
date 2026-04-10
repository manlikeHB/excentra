use rust_decimal::Decimal;
use sqlx::PgPool;
use tokio::sync::broadcast;
use tokio::time::{Duration, sleep};

use crate::error::AppError;
use crate::types::asset_symbol::AssetSymbol;
use crate::utils::ticker::get_ticker_helper;
use crate::{db::queries as db_queries, ws::messages::WsEvent};

pub struct TickerService {
    pool: PgPool,
    ws_sender: broadcast::Sender<WsEvent>,
}

#[derive(Debug, serde::Serialize)]
pub struct Ticker {
    pub symbol: String,
    pub last_price: Decimal,
    pub high_24h: Decimal,
    pub low_24h: Decimal,
    pub volume_24h: Decimal,
    pub price_change_pct: Decimal,
}

impl Ticker {
    pub fn new(
        symbol: &str,
        last_price: Decimal,
        high_24h: Decimal,
        low_24h: Decimal,
        volume_24h: Decimal,
        price_change_pct: Decimal,
    ) -> Self {
        Ticker {
            symbol: symbol.to_string(),
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

            let trade_stats = match db_queries::get_all_trade_stats(&self.pool).await {
                Ok(ts) => ts,
                Err(_) => continue,
            };

            for stat in trade_stats {
                // if there is no ticker — skip
                let ticker = match get_ticker_helper(&stat) {
                    Some(t) => t,
                    None => continue,
                };

                let _ = self.ws_sender.send(WsEvent::TickerUpdate {
                    symbol: stat.symbol,
                    last_price: ticker.last_price,
                    high_24h: ticker.high_24h,
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

        let ts = match db_queries::get_trade_stats(&self.pool, tp.id).await? {
            Some(trade_stat) => trade_stat,
            None => {
                return Err(AppError::NotFound(format!(
                    "No trades found for this pair {}",
                    symbol.as_str()
                )));
            }
        };

        let ticker = match get_ticker_helper(&ts) {
            Some(t) => t,
            None => {
                return Err(AppError::NotFound(format!(
                    "No trades found for this pair {}",
                    ts.symbol
                )));
            }
        };

        Ok(ticker)
    }

    pub async fn get_all_tickers(&self) -> Result<Vec<Ticker>, AppError> {
        let mut tickers = vec![];

        let trade_stats = db_queries::get_all_trade_stats(&self.pool).await?;

        for ts in trade_stats {
            match get_ticker_helper(&ts) {
                Some(t) => tickers.push(t),
                None => {
                    tracing::warn!(symbol = %ts.symbol, "No trades found for");
                }
            };
        }

        tracing::info!(count = %tickers.len(), "Tickers fetched");
        Ok(tickers)
    }
}
