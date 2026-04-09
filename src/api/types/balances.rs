use rust_decimal::Decimal;

use crate::{db::models::balance::DBBalance, error::AppError};

#[derive(Debug, serde::Deserialize)]
pub struct BalanceRequest {
    pub amount: Decimal,
    pub asset: String,
}

impl BalanceRequest {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.amount < Decimal::ZERO {
            return Err(AppError::BadRequest(
                "Amount cannot be less than Zero".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, serde::Serialize)]
pub struct BalanceResponse {
    pub asset: String,
    pub available: Decimal,
    pub held: Decimal,
}

impl From<DBBalance> for BalanceResponse {
    fn from(bal: DBBalance) -> Self {
        BalanceResponse {
            asset: bal.asset,
            available: bal.available,
            held: bal.held,
        }
    }
}
