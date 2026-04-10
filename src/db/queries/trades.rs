use sqlx::PgPool;
use uuid::Uuid;

use crate::db::models::trade::{DBTrade, LastPrice, TradeStat};

pub async fn create_trade<'e, E>(executor: E, trade: DBTrade) -> Result<DBTrade, sqlx::Error>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as!(
        DBTrade,
        r#"
    INSERT INTO trades (pair_id, buy_order_id, sell_order_id, price, quantity)
    VALUES ($1, $2, $3, $4, $5) 
    RETURNING *
    "#,
        trade.pair_id,
        trade.buy_order_id,
        trade.sell_order_id,
        trade.price,
        trade.quantity
    )
    .fetch_one(executor)
    .await
}

pub async fn get_recent_trades(
    pool: &PgPool,
    pair_id: Uuid,
    limit: i64,
) -> Result<Vec<DBTrade>, sqlx::Error> {
    sqlx::query_as!(
        DBTrade,
        r#"
    SELECT * FROM trades 
    WHERE pair_id = $1
    ORDER BY created_at DESC
    LIMIT $2
    "#,
        pair_id,
        limit
    )
    .fetch_all(pool)
    .await
}

pub async fn get_trade_stats(
    pool: &PgPool,
    pair_id: Uuid,
) -> Result<Option<TradeStat>, sqlx::Error> {
    sqlx::query_as!(
        TradeStat,
        r#"
            SELECT 
                tp.symbol,
                MAX(t.price) as high_24h,
                MIN(t.price) as low_24h,
                SUM(t.quantity) as volume_24h,
                (SELECT price FROM trades WHERE pair_id = $1 AND created_at >= NOW() - INTERVAL '24 hours'      ORDER BY created_at ASC LIMIT 1) as oldest_price,
                (SELECT price FROM trades WHERE pair_id = $1 AND created_at < NOW() - INTERVAL '24 hours'       ORDER BY created_at DESC LIMIT 1) as baseline_price,
                (SELECT price FROM trades WHERE pair_id = $1 ORDER BY created_at DESC LIMIT 1) as last_price
            FROM trades t
            JOIN trading_pairs tp ON tp.id = t.pair_id
            WHERE t.pair_id = $1 AND tp.is_active = true
            AND t.created_at >= NOW() - INTERVAL '24 hours'
            GROUP BY tp.symbol
    "#,
        pair_id
    )
    .fetch_optional(pool)
    .await
}

pub async fn get_last_trade_price(
    pool: &PgPool,
    pair_id: Uuid,
) -> Result<Option<LastPrice>, sqlx::Error> {
    sqlx::query_as!(
        LastPrice,
        r#"
        SELECT price FROM trades
        WHERE pair_id = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
        pair_id
    )
    .fetch_optional(pool)
    .await
}

pub async fn get_baseline_trade(
    pool: &PgPool,
    pair_id: Uuid,
) -> Result<Option<DBTrade>, sqlx::Error> {
    sqlx::query_as!(DBTrade, r#"SELECT * FROM trades WHERE pair_id = $1 AND created_at < NOW() - INTERVAL '24 hours' ORDER BY created_at DESC LIMIT 1"#, pair_id).fetch_optional(pool).await
}

pub async fn get_all_trade_stats(pool: &PgPool) -> Result<Vec<TradeStat>, sqlx::Error> {
    sqlx::query_as!(
        TradeStat,
        r#"
            SELECT 
                tp.symbol,
                MAX(t.price) as high_24h,
                MIN(t.price) as low_24h,
                SUM(t.quantity) as volume_24h,
                (SELECT t2.price FROM trades t2 
                WHERE t2.pair_id = tp.id 
                AND t2.created_at >= NOW() - INTERVAL '24 hours' 
                ORDER BY t2.created_at ASC LIMIT 1) as oldest_price,
                (SELECT t2.price FROM trades t2 
                WHERE t2.pair_id = tp.id 
                AND t2.created_at < NOW() - INTERVAL '24 hours' 
                ORDER BY t2.created_at DESC LIMIT 1) as baseline_price,
                (SELECT t2.price FROM trades t2 
                WHERE t2.pair_id = tp.id 
                ORDER BY t2.created_at DESC LIMIT 1) as last_price
            FROM trading_pairs tp
            LEFT JOIN trades t ON t.pair_id = tp.id 
                AND t.created_at >= NOW() - INTERVAL '24 hours'
            WHERE tp.is_active = true
            GROUP BY tp.id, tp.symbol
    "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn count_trades(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let count = sqlx::query_scalar!("SELECT COUNT(*) FROM trades")
        .fetch_one(pool)
        .await?
        .unwrap_or(0);
    Ok(count)
}
