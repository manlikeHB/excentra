pub mod admin;
pub mod app_state;
pub mod asset;
pub mod auth;
pub mod balances;
pub mod health;
pub mod order;
pub mod orderbook;
pub mod password_reset;
pub mod ticker;
pub mod trades;
pub mod trading_pairs;
pub mod users;

pub use app_state::AppState;

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub page: u64,
    pub limit: u64,
    pub total: i64,
}
