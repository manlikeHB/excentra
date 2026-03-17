use sqlx::PgPool;
use uuid::Uuid;

use crate::db::models::trade::DBTrade;

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

pub async fn get_trades_from_last_24_hours(
    pool: &PgPool,
    pair_id: Uuid,
) -> Result<Vec<DBTrade>, sqlx::Error> {
    sqlx::query_as!(DBTrade, r#"SELECT * FROM trades WHERE pair_id = $1 AND created_at >= NOW() - INTERVAL '24 hours' ORDER BY created_at DESC"#, pair_id).fetch_all(pool).await
}

pub async fn get_baseline_trade(
    pool: &PgPool,
    pair_id: Uuid,
) -> Result<Option<DBTrade>, sqlx::Error> {
    sqlx::query_as!(DBTrade, r#"SELECT * FROM trades WHERE pair_id = $1 AND created_at < NOW() - INTERVAL '24 hours' ORDER BY created_at DESC LIMIT 1"#, pair_id).fetch_optional(pool).await
}
