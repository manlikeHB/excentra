use std::sync::Arc;

use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal_macros::dec;
use sqlx::PgPool;
use std::collections::HashMap;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::constants;
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
        let existing =
            db_queries::get_open_orders_by_user(&self.pool, constants::SYSTEM_USER_ID).await?;
        if !existing.is_empty() {
            return Ok(());
        }

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
                    Ok(p) => {
                        tracing::info!(pair = %pair.base_asset, price = %p, "Price fetched from CoinGecko");
                        p
                    }
                    Err(_) => {
                        tracing::warn!(pair = %pair.base_asset, "CoinGecko fetch failed, using hardcoded fallback");

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
        let (spread, levels, base_qty) = match pair.base_asset.as_str() {
            "BTC" => (dec!(0.001), 5, dec!(0.1)),
            "ETH" => (dec!(0.001), 5, dec!(1.5)),
            "SOL" => (dec!(0.002), 8, dec!(15.0)),
            _ => (dec!(0.001), 3, dec!(1.0)),
        };

        let mut bids = vec![];
        let mut asks = vec![];

        for i in 1..=levels {
            let i_dec = Decimal::from(i);
            let qty = base_qty * i_dec;
            bids.push((price * (Decimal::ONE - spread * i_dec), qty));
            asks.push((price * (Decimal::ONE + spread * i_dec), qty));
        }

        let mut tx = self.pool.begin().await?;

        for (bid_price, quantity) in bids {
            let mut order = Order::new(
                Uuid::new_v4(),
                constants::SYSTEM_USER_ID, // system user
                pair.id,
                OrderSide::Buy,
                OrderType::Limit,
                Some(bid_price),
                quantity,
                quantity,
            );

            // hold quote asset
            db_queries::hold(
                &mut *tx,
                constants::SYSTEM_USER_ID,
                &pair.quote_asset,
                bid_price * quantity,
            )
            .await?;

            // place order
            self.exchange
                .lock()
                .await
                .place_order(pair.id, &mut order)?;

            // persist order in db
            db_queries::create_order(&mut *tx, order.into()).await?;
        }

        for (ask_price, quantity) in asks {
            let mut order = Order::new(
                Uuid::new_v4(),
                constants::SYSTEM_USER_ID,
                pair.id,
                OrderSide::Sell,
                OrderType::Limit,
                Some(ask_price),
                quantity,
                quantity,
            );

            // hold base asset
            db_queries::hold(
                &mut *tx,
                constants::SYSTEM_USER_ID,
                &pair.base_asset,
                quantity,
            )
            .await?;

            // place order
            self.exchange
                .lock()
                .await
                .place_order(pair.id, &mut order)?;

            // persist order in db
            db_queries::create_order(&mut *tx, order.into()).await?;
        }

        tx.commit().await?;

        Ok(())
    }
}
