use crate::services::ticker::Ticker;
use rust_decimal::Decimal;

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct TickerResponse {
    pub symbol: String,
    pub last_price: Decimal,
    pub high_24h: Decimal,
    pub low_24h: Decimal,
    pub volume_24h: Decimal,
    pub price_change_pct: Decimal,
}

impl From<Ticker> for TickerResponse {
    fn from(t: Ticker) -> Self {
        TickerResponse {
            symbol: t.symbol,
            last_price: t.last_price,
            high_24h: t.high_24h,
            low_24h: t.low_24h,
            volume_24h: t.volume_24h,
            price_change_pct: t.price_change_pct,
        }
    }
}
