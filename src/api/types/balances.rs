use std::str::FromStr;

use rust_decimal::Decimal;

use crate::{db::models::balance::DBBalance, error::AppError};

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
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

    // Deposit cap per asset.
    //
    // These limits exist to prevent users from inflating their balances
    // arbitrarily on the simulated deposit system — without a cap, anyone
    // could deposit Decimal::MAX in a single request, bypassing the rate limiter.
    //
    // Drawback: hardcoded here and in the frontend.
    // Adding a new asset requires updating both places manually, which is
    // error-prone and inconsistent with Excentra's dynamic asset design.
    // A proper fix would be a `deposit_cap` column on the `assets` table,
    // queried at the service layer — new assets would carry their own cap
    // automatically.
    //
    // This is intentionally temporary. Once real blockchain integration lands
    // deposits will be on-chain transactions — the exchange credits
    // only what it observes confirmed on the blockchain. At that point, a
    // simulated cap makes no sense and this validation gets removed entirely.
    pub fn validate_deposit(&self) -> Result<(), AppError> {
        if self.amount <= Decimal::ZERO {
            return Err(AppError::BadRequest(
                "Amount must be greater than zero".to_string(),
            ));
        }

        let max = match self.asset.to_uppercase().as_str() {
            "USDT" => Decimal::from(1000),
            "BTC" => Decimal::from_str("0.05").unwrap(),
            "ETH" => Decimal::from_str("0.5").unwrap(),
            "SOL" => Decimal::from(5),
            _ => return Ok(()), // unknown asset handled downstream by is_valid_asset check
        };

        if self.amount > max {
            return Err(AppError::BadRequest(format!(
                "Maximum deposit for {} is {}",
                self.asset.to_uppercase(),
                max
            )));
        }

        Ok(())
    }
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema, serde::Deserialize)]
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
