use rust_decimal::Decimal;
use sqlx::{PgPool, Transaction};
use uuid::Uuid;

use crate::{
    api::types::order::OrderResponse,
    db::models::order::{DBOrder, DBOrderSide, DBOrderStatus, DBOrderType},
};

pub async fn create_order<'e, E>(executor: E, order: DBOrder) -> Result<DBOrder, sqlx::Error>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as!(
        DBOrder,
        r#"INSERT INTO orders 
    (id, user_id, pair_id, side, order_type, price, quantity, remaining_quantity, status) 
    VALUES ($9, $1, $2, $3::order_side, $4::order_type, $5, $6, $7, $8::order_status) 
    RETURNING 
    id, 
    user_id, pair_id, 
    side as "side: DBOrderSide", 
    order_type as "order_type: DBOrderType", 
    price, quantity, 
    remaining_quantity, 
    status as "status: DBOrderStatus", 
    created_at, 
    updated_at"#,
        order.user_id,
        order.pair_id,
        order.side as DBOrderSide,
        order.order_type as DBOrderType,
        order.price,
        order.quantity,
        order.remaining_quantity,
        order.status as DBOrderStatus,
        order.id,
    )
    .fetch_one(executor)
    .await
}

pub async fn update_order_status(
    pool: &PgPool,
    order_id: Uuid,
    status: DBOrderStatus,
) -> Result<Option<DBOrder>, sqlx::Error> {
    sqlx::query_as!(
        DBOrder,
        r#"UPDATE orders SET status = $1, updated_at = NOW() WHERE id = $2 
    RETURNING 
    id, 
    user_id, pair_id, 
    side as "side: DBOrderSide", 
    order_type as "order_type: DBOrderType", 
    price, quantity, 
    remaining_quantity, 
    status as "status: DBOrderStatus", 
    created_at, 
    updated_at"#,
        status as DBOrderStatus,
        order_id
    )
    .fetch_optional(pool)
    .await
}

pub async fn get_user_orders(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<OrderResponse>, sqlx::Error> {
    sqlx::query_as!(
        OrderResponse,
        r#"SELECT o.id, tp.symbol, o.side as "side: DBOrderSide", 
        o.order_type as "order_type: DBOrderType", o.price, o.quantity, 
        o.remaining_quantity, o.status as "status: DBOrderStatus", 
        o.created_at, o.updated_at
        FROM orders o
        JOIN trading_pairs tp ON o.pair_id = tp.id
        WHERE o.user_id = $1
        ORDER BY o.created_at DESC"#,
        user_id
    )
    .fetch_all(pool)
    .await
}

pub async fn get_order_by_id(
    pool: &PgPool,
    order_id: Uuid,
) -> Result<Option<DBOrder>, sqlx::Error> {
    sqlx::query_as!(DBOrder, r#"
    SELECT id, user_id, pair_id, side as "side: DBOrderSide", order_type as "order_type: DBOrderType", price, quantity, remaining_quantity, status as "status: DBOrderStatus", created_at, updated_at FROM orders 
    WHERE id = $1"#, order_id).fetch_optional(pool).await
}

pub async fn update_order_after_trade(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    order_id: Uuid,
    status: DBOrderStatus,
    remaining_quantity: Decimal,
) -> Result<Option<DBOrder>, sqlx::Error> {
    sqlx::query_as!(
        DBOrder,
        r#"UPDATE orders SET status = $2, remaining_quantity = $3, updated_at = NOW() WHERE id = $1
    RETURNING 
    id, 
    user_id, pair_id, 
    side as "side: DBOrderSide", 
    order_type as "order_type: DBOrderType", 
    price, quantity, 
    remaining_quantity, 
    status as "status: DBOrderStatus", 
    created_at, 
    updated_at"#,
        order_id,
        status as DBOrderStatus,
        remaining_quantity
    )
    .fetch_optional(&mut **tx)
    .await
}

pub async fn get_open_orders_by_pair(
    pool: &PgPool,
    pair_id: Uuid,
) -> Result<Vec<DBOrder>, sqlx::Error> {
    sqlx::query_as!(
        DBOrder,
        r#"SELECT id, user_id, pair_id, 
        side as "side: DBOrderSide", 
        order_type as "order_type: DBOrderType", 
        price, quantity, remaining_quantity, 
        status as "status: DBOrderStatus", 
        created_at, updated_at 
        FROM orders 
        WHERE pair_id = $1 
        AND status IN ('open', 'partially_filled')
        ORDER BY created_at ASC"#,
        pair_id
    )
    .fetch_all(pool)
    .await
}
