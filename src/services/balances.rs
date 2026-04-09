use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    db::{models::balance::DBBalance, queries as db_queries},
    error::AppError,
};

pub struct BalanceService {
    pool: PgPool,
}

impl BalanceService {
    pub fn new(pool: PgPool) -> Self {
        BalanceService { pool }
    }

    // TODO: Impl real deposit from supported blockchain
    pub async fn deposit(
        &self,
        user_id: Uuid,
        amount: Decimal,
        asset: &str,
    ) -> Result<DBBalance, AppError> {
        // verify asset is supported
        if !db_queries::is_valid_asset(&self.pool, asset).await? {
            tracing::warn!(user_id = %user_id, asset = %asset, "Deposit of non supported asset");
            return Err(AppError::Unprocessable(format!(
                "{} is not supported",
                asset
            )));
        }

        let bal = db_queries::deposit(&self.pool, user_id, asset, amount).await?;

        tracing::info!(user_id = %user_id, asset = %asset, amount = %amount, "Deposit credited");

        Ok(bal)
    }

    pub async fn get_balances(&self, user_id: Uuid) -> Result<Vec<DBBalance>, AppError> {
        Ok(db_queries::get_balances(&self.pool, user_id).await?)
    }

    // TODO: Impl real withdrawal from supported blockchain
    pub async fn withdraw(
        &self,
        user_id: Uuid,
        amount: Decimal,
        asset: &str,
    ) -> Result<DBBalance, AppError> {
        // verify asset is supported
        if !db_queries::is_valid_asset(&self.pool, asset).await? {
            tracing::warn!(user_id = %user_id, asset = %asset, "withdrawal of non supported asset");
            return Err(AppError::Unprocessable(format!(
                "{} is not supported",
                asset
            )));
        }

        let bal = match db_queries::withdraw(&self.pool, user_id, asset, amount).await? {
            Some(b) => b,
            None => {
                tracing::warn!(user_id = %user_id, asset = %asset, amount = %amount, "Insufficient balance withdrawal");
                return Err(AppError::Unprocessable("Insufficient balance".to_string()));
            }
        };

        tracing::info!(user_id = %user_id, asset = %asset, amount = %amount, "Withdraw successful");

        Ok(bal)
    }

    pub async fn get_balance(&self, user_id: Uuid, asset: &str) -> Result<DBBalance, AppError> {
        if !db_queries::is_valid_asset(&self.pool, asset).await? {
            return Err(AppError::Unprocessable(format!(
                "{} is not supported",
                asset
            )));
        }

        let balance = match db_queries::get_balance(&self.pool, user_id, asset).await? {
            Some(b) => b,
            // Zero balance
            None => DBBalance {
                id: Uuid::nil(), // synthetic — not persisted
                user_id,
                asset: asset.to_string(),
                available: Decimal::ZERO,
                held: Decimal::ZERO,
                updated_at: chrono::Utc::now(),
            },
        };

        Ok(balance)
    }
}
