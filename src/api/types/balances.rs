use rust_decimal::Decimal;
use uuid::Uuid;

use crate::db::models::balance::Balance;

#[derive(Debug, serde::Deserialize)]
pub struct BalanceRequest {
    pub amount: Decimal,
    pub asset: String,
}

#[derive(Debug, serde::Serialize)]
pub struct BalanceResponse {
    pub user_id: Uuid,
    pub asset: String,
    pub available: Decimal,
    pub held: Decimal,
}

impl From<Balance> for BalanceResponse {
    fn from(bal: Balance) -> Self {
        BalanceResponse {
            user_id: bal.user_id,
            asset: bal.asset,
            available: bal.available,
            held: bal.held,
        }
    }
}
