use sqlx::PgPool;
use uuid::Uuid;

use crate::db::models::trading_pairs::DBTradingPair;

pub async fn find_by_symbol(
    pool: &PgPool,
    symbol: &str,
) -> Result<Option<DBTradingPair>, sqlx::Error> {
    sqlx::query_as!(
        DBTradingPair,
        r#"SELECT * FROM trading_pairs WHERE symbol = $1"#,
        symbol
    )
    .fetch_optional(pool)
    .await
}

pub async fn find_trading_pair_by_id(
    pool: &PgPool,
    pair_id: Uuid,
) -> Result<Option<DBTradingPair>, sqlx::Error> {
    sqlx::query_as!(
        DBTradingPair,
        r#"SELECT * FROM trading_pairs WHERE id = $1"#,
        pair_id
    )
    .fetch_optional(pool)
    .await
}

pub async fn get_all_trading_pairs(pool: &PgPool) -> Result<Vec<DBTradingPair>, sqlx::Error> {
    sqlx::query_as!(DBTradingPair, r#"SELECT * FROM trading_pairs"#)
        .fetch_all(pool)
        .await
}

pub async fn get_active_trading_pairs(pool: &PgPool) -> Result<Vec<DBTradingPair>, sqlx::Error> {
    sqlx::query_as!(
        DBTradingPair,
        r#"SELECT * FROM trading_pairs WHERE is_active = true"#
    )
    .fetch_all(pool)
    .await
}

pub async fn add_trading_pair(
    pool: &PgPool,
    base_asset: &str,
    quote_asset: &str,
) -> Result<DBTradingPair, sqlx::Error> {
    let symbol = format!("{}/{}", base_asset, quote_asset);
    sqlx::query_as!(
        DBTradingPair,
        r#"
    INSERT INTO trading_pairs (base_asset, quote_asset, symbol) 
    VALUES ($1,$2, $3) RETURNING *"#,
        base_asset,
        quote_asset,
        symbol
    )
    .fetch_one(pool)
    .await
}
