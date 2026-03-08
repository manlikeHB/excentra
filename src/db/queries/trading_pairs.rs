use sqlx::PgPool;

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
