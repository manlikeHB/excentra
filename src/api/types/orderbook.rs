use crate::engine::models::orderbook::PriceLevel;
use rust_decimal::Decimal;

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct OrderBookParams {
    pub levels: Option<usize>,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct OrderBookResponse {
    pub symbol: String,
    pub bids: Vec<PriceLevelResponse>,
    pub asks: Vec<PriceLevelResponse>,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct PriceLevelResponse {
    pub price: Decimal,
    pub quantity: Decimal,
}

impl From<PriceLevel> for PriceLevelResponse {
    fn from(pl: PriceLevel) -> Self {
        PriceLevelResponse {
            price: pl.price(),
            quantity: pl.quantity(),
        }
    }
}
