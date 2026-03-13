use sqlx::PgPool;

use crate::{
    db::{models::trade::DBTrade, queries as db_queries},
    error::AppError,
    types::asset::{AssetSymbol, AssetSymbolError},
};

pub struct TradeService {
    pool: PgPool,
}

impl TradeService {
    pub fn new(pool: PgPool) -> Self {
        TradeService { pool }
    }

    pub async fn get_trades(
        &self,
        asset_symbol: &AssetSymbol,
        limit: i64,
    ) -> Result<Vec<DBTrade>, AppError> {
        // get pair ID
        let tp = db_queries::find_by_symbol(&self.pool, asset_symbol.as_str())
            .await?
            .ok_or(AssetSymbolError::MarketNotSupported(
                asset_symbol.as_str().to_string(),
            ))?;

        let trades = db_queries::get_recent_trades(&self.pool, tp.id, limit).await?;

        Ok(trades)
    }
}
