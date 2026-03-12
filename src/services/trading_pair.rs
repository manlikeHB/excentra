use std::sync::Arc;

use sqlx::PgPool;
use tokio::sync::Mutex;

use crate::db::models::trading_pairs::DBTradingPair;
use crate::db::queries as db_queries;
use crate::error::AppError;
use crate::{api::types::trading_pairs::TradingPairsResponse, engine::exchange::Exchange};

pub struct TradingPairService {
    pool: PgPool,
    exchange: Arc<Mutex<Exchange>>,
}

impl TradingPairService {
    pub fn new(pool: PgPool, exchange: Arc<Mutex<Exchange>>) -> Self {
        TradingPairService { pool, exchange }
    }

    pub async fn get_active_trading_pairs(&self) -> Result<Vec<TradingPairsResponse>, AppError> {
        let res = db_queries::get_active_trading_pairs(&self.pool)
            .await?
            .into_iter()
            .map(TradingPairsResponse::from)
            .collect();

        Ok(res)
    }

    pub async fn get_all_trading_pairs(&self) -> Result<Vec<TradingPairsResponse>, AppError> {
        let res = db_queries::get_all_trading_pairs(&self.pool)
            .await?
            .into_iter()
            .map(TradingPairsResponse::from)
            .collect();

        Ok(res)
    }

    pub async fn add_trading_pair(
        &self,
        base_asset: &str,
        quote_asset: &str,
    ) -> Result<DBTradingPair, AppError> {
        // Check if asset is supported
        if !db_queries::is_valid_asset(&self.pool, base_asset).await? {
            return Err(AppError::BadRequest(format!(
                "Asset is not supported: {}",
                base_asset
            )));
        }

        if !db_queries::is_valid_asset(&self.pool, quote_asset).await? {
            return Err(AppError::BadRequest(format!(
                "Asset is not supported: {}",
                quote_asset
            )));
        }

        // add trading pair to DB
        let res = db_queries::add_trading_pair(&self.pool, base_asset, quote_asset).await?;

        // add trading pair to exchange
        self.exchange.lock().await.add_trading_pair(res.id);
        Ok(res)
    }
}
