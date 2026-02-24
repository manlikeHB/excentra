use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TradingPair {
    id: Uuid,
    base_asset: String,
    quote_asset: String,
    symbol: String,
    is_active: bool,
}
