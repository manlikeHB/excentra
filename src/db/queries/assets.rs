use crate::db::models::assets::Asset;
use sqlx::PgPool;

pub async fn is_valid_asset(pool: &PgPool, symbol: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT EXISTS(
            SELECT 1 FROM assets WHERE symbol = $1 AND is_active = true
        ) as "exists!""#,
        symbol
    )
    .fetch_one(pool)
    .await?;

    Ok(result.exists)
}

pub async fn get_all_assets(pool: &PgPool) -> Result<Vec<Asset>, sqlx::Error> {
    sqlx::query_as!(Asset, r#"SELECT * FROM assets"#)
        .fetch_all(pool)
        .await
}

pub async fn add_asset(
    pool: &PgPool,
    symbol: &str,
    name: &str,
    decimals: i16,
) -> Result<Asset, sqlx::Error> {
    sqlx::query_as!(
        Asset,
        r#"INSERT INTO assets (symbol, name, decimals) VALUES ($1, $2, $3) RETURNING *"#,
        symbol,
        name,
        decimals
    )
    .fetch_one(pool)
    .await
}
