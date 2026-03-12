use sqlx::PgPool;

use crate::{
    db::{models::assets::Asset, queries as db_queries},
    error::AppError,
};

pub struct AssetService {
    pool: PgPool,
}

impl AssetService {
    pub fn new(pool: PgPool) -> Self {
        AssetService { pool }
    }

    pub async fn add_asset(
        &self,
        symbol: &str,
        name: &str,
        decimals: i16,
    ) -> Result<Asset, AppError> {
        Ok(db_queries::add_asset(&self.pool, symbol, name, decimals).await?)
    }

    pub async fn get_all_asset(&self) -> Result<Vec<Asset>, AppError> {
        Ok(db_queries::get_all_assets(&self.pool).await?)
    }
}
