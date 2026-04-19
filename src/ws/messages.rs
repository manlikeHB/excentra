use crate::db::models::order::DBOrderSide;
use crate::{
    db::models::order::DBOrderStatus, engine::models::orderbook::OrderBookSnapshot,
    types::asset_symbol::AssetSymbol,
};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::fmt;
use uuid::Uuid;

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "action", rename_all = "lowercase")]
pub enum InboundMessage {
    Subscribe { channel: String },
    Unsubscribe { channel: String },
    Auth { token: String },
}

#[derive(Debug, serde::Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum OutboundMessage {
    Subscribed { channel: String },
    Unsubscribed { channel: String },
    Error { message: String },
    Event { data: WsEvent },
    Authenticated,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum WsEvent {
    OrderBookUpdate {
        symbol: String,
        snapshot: OrderBookSnapshot,
    },
    TradeEvent {
        symbol: String,
        price: Decimal,
        quantity: Decimal,
        side: DBOrderSide,
        created_at: DateTime<Utc>,
    },
    OrderStatusUpdate {
        user_id: Uuid,
        order_id: Uuid,
        status: DBOrderStatus,
        quantity: Decimal,
        remaining_quantity: Decimal,
    },
    TickerUpdate {
        symbol: String,
        last_price: Decimal,
        high_24h: Decimal,
        low_24h: Decimal,
        volume_24h: Decimal,
        price_change_pct: Decimal,
    },
}

pub enum Channel {
    OrderBook(String),
    Trades(String),
    Ticker(String),
    Orders(Uuid), // protected
}

impl Channel {
    pub fn from_str(channel: &str) -> Result<Self, String> {
        let parts: Vec<&str> = channel.splitn(2, ":").collect();
        if parts.len() != 2 {
            tracing::warn!(channel = %channel, "Invalid ws channel");
            return Err("Invalid ws channel e.g `trades:BTC-USDT`".to_string());
        }

        match parts[0] {
            "orderbook" => Ok(Channel::OrderBook(
                AssetSymbol::new(parts[1])?.as_str().to_string(),
            )),
            "trades" => Ok(Channel::Trades(
                AssetSymbol::new(parts[1])?.as_str().to_string(),
            )),
            "ticker" => Ok(Channel::Ticker(
                AssetSymbol::new(parts[1])?.as_str().to_string(),
            )),
            "orders" => {
                let user_id = Uuid::parse_str(parts[1])
                    .map_err(|_| format!("Invalid user ID in orders channel: {}", parts[1]))?;
                Ok(Channel::Orders(user_id))
            }
            _ => return Err("Unsupported Channel".to_string()),
        }
    }
}

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Channel::OrderBook(symbol) => write!(f, "orderbook:{}", symbol),
            Channel::Trades(symbol) => write!(f, "trades:{}", symbol),
            Channel::Ticker(symbol) => write!(f, "ticker:{}", symbol),
            Channel::Orders(user_id) => write!(f, "orders:{}", user_id),
        }
    }
}
