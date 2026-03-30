use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::engine::models::trade::Trade;

#[derive(Debug, Clone, Copy, sqlx::FromRow)]
pub struct DBTrade {
    pub id: Uuid,
    pub pair_id: Uuid,
    pub buy_order_id: Uuid,
    pub sell_order_id: Uuid,
    pub price: Decimal,
    pub quantity: Decimal,
    pub created_at: DateTime<Utc>,
}

impl From<Trade> for DBTrade {
    fn from(trade: Trade) -> Self {
        DBTrade {
            id: trade.id(),
            pair_id: trade.pair_id(),
            buy_order_id: trade.buy_order_id(),
            sell_order_id: trade.sell_order_id(),
            price: trade.price(),
            quantity: trade.quantity(),
            created_at: trade.created_at(),
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct TradeStat {
    pub high_24h: Option<Decimal>,
    pub low_24h: Option<Decimal>,
    pub volume_24h: Option<Decimal>,
    pub oldest_price: Option<Decimal>,
    pub baseline_price: Option<Decimal>,
    pub last_price: Option<Decimal>,
}

pub struct LastPrice {
    pub price: Decimal,
}
