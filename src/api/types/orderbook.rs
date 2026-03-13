use crate::engine::models::orderbook::PriceLevel;

#[derive(Debug, serde::Deserialize)]
pub struct OrderBookParams {
    pub levels: Option<usize>,
}

#[derive(Debug, serde::Serialize)]
pub struct OrderBookResponse {
    pub symbol: String,
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
}
