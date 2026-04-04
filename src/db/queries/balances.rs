use crate::db::models::balance::Balance;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn get_balances(pool: &PgPool, user_id: Uuid) -> Result<Vec<Balance>, sqlx::Error> {
    sqlx::query_as!(
        Balance,
        r#"SELECT id, user_id, asset, available, held, updated_at FROM balances WHERE user_id = $1"#,
        user_id
    )
    .fetch_all(pool)
    .await
}

pub async fn get_balance(
    pool: &PgPool,
    user_id: Uuid,
    asset: &str,
) -> Result<Option<Balance>, sqlx::Error> {
    sqlx::query_as!(Balance, r#"SELECT id, user_id, asset, available, held, updated_at FROM balances WHERE user_id = $1 AND asset = $2"#, user_id, asset).fetch_optional(pool).await
}

pub async fn deposit(
    pool: &PgPool,
    user_id: Uuid,
    asset: &str,
    amount: Decimal,
) -> Result<Balance, sqlx::Error> {
    sqlx::query_as!(
        Balance,
        r#"INSERT INTO balances (user_id, asset, available)
VALUES ($1, $2, $3)
ON CONFLICT (user_id, asset)
DO UPDATE SET available = balances.available + $3, updated_at = NOW()
RETURNING id, user_id, asset, available, held, updated_at"#,
        user_id,
        asset,
        amount
    )
    .fetch_one(pool)
    .await
}

pub async fn hold<'e, E>(
    executor: E,
    user_id: Uuid,
    asset: &str,
    amount: Decimal,
) -> Result<Balance, sqlx::Error>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as!(Balance, r#"UPDATE balances SET available = available - $3, held = held + $3, updated_at = NOW() WHERE user_id = $1 AND asset = $2 AND available >= $3 RETURNING id, user_id, asset, available, held, updated_at"#, user_id, asset, amount).fetch_one(executor).await
}

pub async fn release<'e, E>(
    executor: E,
    user_id: Uuid,
    asset: &str,
    amount: Decimal,
) -> Result<Balance, sqlx::Error>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as!(Balance, r#"UPDATE balances SET available = available + $3, held = held - $3, updated_at = NOW() WHERE user_id = $1 AND asset = $2 AND held >= $3 RETURNING id, user_id, asset, available, held, updated_at"#, user_id, asset, amount).fetch_one(executor).await
}

pub async fn transfer_on_fill(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    buyer_id: Uuid,
    seller_id: Uuid,
    base_asset: &str,
    quote_asset: &str,
    qty: Decimal,
    price: Decimal,
    quote_precision: u32,
) -> Result<(), sqlx::Error> {
    // round cost to the quote asset decimal precision to avoid negative balance
    let cost = (qty * price).round_dp(quote_precision);

    // buyer - quote bal
    sqlx::query!(r#"UPDATE balances SET held = held - $1, updated_at = NOW() WHERE user_id = $2 AND asset = $3"#, cost, buyer_id, quote_asset).execute(&mut **tx).await?;

    // buyer - base bal
    sqlx::query!(
        r#"
    INSERT INTO balances (user_id, asset, available) 
    VALUES ($1, $2, $3) 
    ON CONFLICT (user_id, asset) 
    DO UPDATE SET available = balances.available + $3, updated_at = NOW()
    "#,
        buyer_id,
        base_asset,
        qty
    )
    .execute(&mut **tx)
    .await?;

    // seller - base bal
    sqlx::query!(r#"UPDATE balances SET held = held - $1, updated_at = NOW() WHERE user_id = $2 AND asset = $3"#, qty, seller_id, base_asset).execute(&mut **tx).await?;

    // seller - quote bal
    sqlx::query!(
        r#"
    INSERT INTO balances (user_id, asset, available)
    VALUES ($1, $2, $3) 
    ON CONFLICT (user_id, asset)
    DO UPDATE SET available = balances.available + $3, updated_at = NOW()
    "#,
        seller_id,
        quote_asset,
        cost
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}
