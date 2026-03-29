use std::sync::Arc;

use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal_macros::dec;
use sqlx::PgPool;
use std::collections::HashMap;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::db::models::trading_pairs::DBTradingPair;
use crate::db::queries as db_queries;
use crate::engine::models::order::{Order, OrderSide, OrderType};
use crate::{engine::exchange::Exchange, error::AppError};

pub struct PriceSeedService {
    pub pool: PgPool,
    pub exchange: Arc<Mutex<Exchange>>,
    pub client: reqwest::Client,
}

#[derive(serde::Deserialize)]
struct CoinGeckoPrice {
    usd: f64,
}

impl PriceSeedService {
    pub fn new(pool: PgPool, exchange: Arc<Mutex<Exchange>>, client: reqwest::Client) -> Self {
        PriceSeedService {
            pool,
            exchange,
            client,
        }
    }

    pub async fn fetch_from_coingecko(&self, coingecko_id: &str) -> Result<Decimal, AppError> {
        let url = format!(
            "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd",
            coingecko_id
        );

        let text = self
            .client
            .get(&url)
            .header("User-Agent", "Excentra-Exchange/1.0")
            .send()
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .text()
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        let res: HashMap<String, CoinGeckoPrice> =
            serde_json::from_str(&text).map_err(|e| AppError::InternalError(e.to_string()))?;

        let price = res
            .get(coingecko_id)
            .ok_or(AppError::InternalError(format!(
                "CoinGecko returned no price for {}",
                coingecko_id
            )))?;

        Decimal::from_f64(price.usd).ok_or(AppError::InternalError(format!(
            "Failed to convert price to Decimal for {}",
            coingecko_id
        )))
    }

    pub async fn seed_prices(&self) -> Result<(), AppError> {
        let assets = db_queries::get_assets_with_coingecko_ids(&self.pool).await?;
        let pairs = db_queries::get_active_trading_pairs(&self.pool).await?;

        for pair in pairs {
            // find the base asset's coingecko_id
            let coingecko_id = assets
                .iter()
                .find(|a| a.symbol == pair.base_asset)
                .and_then(|a| a.coingecko_id.as_deref());

            let price = if let Some(id) = coingecko_id {
                match self.fetch_from_coingecko(id).await {
                    Ok(p) => p,
                    Err(_) => {
                        // hardcoded fallback
                        match pair.base_asset.as_str() {
                            "BTC" => dec!(65000),
                            "ETH" => dec!(3500),
                            "SOL" => dec!(150),
                            _ => continue, // unknown asset, skip
                        }
                    }
                }
            } else {
                continue; // no coingecko_id, skip
            };

            // place resting orders around the current price to seed liquidity
            self.seed_order_book(&pair, price).await?;
        }

        Ok(())
    }

    async fn seed_order_book(&self, pair: &DBTradingPair, price: Decimal) -> Result<(), AppError> {
        let spread = dec!(0.001); // 0.1% spread

        let bids = vec![
            (price * (Decimal::ONE - spread), dec!(0.5)),
            (price * (Decimal::ONE - spread * dec!(2)), dec!(1.0)),
            (price * (Decimal::ONE - spread * dec!(3)), dec!(2.0)),
        ];

        let asks = vec![
            (price * (Decimal::ONE + spread), dec!(0.5)),
            (price * (Decimal::ONE + spread * dec!(2)), dec!(1.0)),
            (price * (Decimal::ONE + spread * dec!(3)), dec!(2.0)),
        ];

        let mut exchange = self.exchange.lock().await;

        for (bid_price, quantity) in bids {
            let mut order = Order::new(
                Uuid::new_v4(),
                Uuid::nil(), // system order — no real user
                pair.id,
                OrderSide::Buy,
                OrderType::Limit,
                Some(bid_price),
                quantity,
                quantity,
            );
            exchange.place_order(pair.id, &mut order)?;
        }

        for (ask_price, quantity) in asks {
            let mut order = Order::new(
                Uuid::new_v4(),
                Uuid::nil(),
                pair.id,
                OrderSide::Sell,
                OrderType::Limit,
                Some(ask_price),
                quantity,
                quantity,
            );
            exchange.place_order(pair.id, &mut order)?;
        }

        Ok(())
    }
}
