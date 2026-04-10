use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    db::{
        models::trade::{DBTrade, TradeWithSymbolAndSide},
        queries as db_queries,
    },
    error::AppError,
    types::asset_symbol::{AssetSymbol, AssetSymbolError},
    utils::query_builder::{self, QueryOrder},
};

pub struct TradeService {
    pool: PgPool,
}

impl TradeService {
    pub fn new(pool: PgPool) -> Self {
        TradeService { pool }
    }

    pub async fn get_trades(
        &self,
        asset_symbol: &AssetSymbol,
        limit: u64,
    ) -> Result<Vec<DBTrade>, AppError> {
        // get pair ID
        let tp = db_queries::find_by_symbol(&self.pool, asset_symbol.as_str())
            .await?
            .ok_or(AssetSymbolError::MarketNotSupported(
                asset_symbol.as_str().to_string(),
            ))?;

        let trades = db_queries::get_recent_trades(&self.pool, tp.id, limit as i64).await?;

        Ok(trades)
    }

    pub async fn get_trade_history(
        &self,
        user_id: Uuid,
        pair: Option<&str>,
        page: Option<u64>,
        limit: Option<u64>,
        order: Option<QueryOrder>,
    ) -> Result<(Vec<TradeWithSymbolAndSide>, i64), AppError> {
        // build trade query
        let mut trade_builder = sqlx::QueryBuilder::new(
            "SELECT t.id, tp.symbol, t.price, t.quantity, t.created_at,
            CASE WHEN bo.user_id = ",
        );
        trade_builder.push_bind(user_id);
        trade_builder.push(
            " THEN 'buy'::order_side ELSE 'sell'::order_side END as side
            FROM trades t
            JOIN trading_pairs tp ON t.pair_id = tp.id
            JOIN orders bo ON bo.id = t.buy_order_id
            JOIN orders so ON so.id = t.sell_order_id
            WHERE (bo.user_id = ",
        );
        trade_builder.push_bind(user_id);
        trade_builder.push(" OR so.user_id = ");
        trade_builder.push_bind(user_id);
        trade_builder.push(")");

        // build count query
        let mut count_builder = sqlx::QueryBuilder::new(
            "SELECT COUNT(*) FROM trades t
            JOIN trading_pairs tp ON t.pair_id = tp.id
            JOIN orders bo ON bo.id = t.buy_order_id
            JOIN orders so ON so.id = t.sell_order_id
            WHERE (bo.user_id = ",
        );
        count_builder.push_bind(user_id);
        count_builder.push(" OR so.user_id = ");
        count_builder.push_bind(user_id);
        count_builder.push(")");

        // apply filter and pagination to trade_builder
        query_builder::apply_pair_filter(&self.pool, &mut trade_builder, pair, "t").await?;
        query_builder::apply_pagination(&mut trade_builder, page, limit, "t", order);

        // apply filter and pagination to count_builder
        query_builder::apply_pair_filter(&self.pool, &mut count_builder, pair, "t").await?;

        let trades: Vec<TradeWithSymbolAndSide> =
            trade_builder.build_query_as().fetch_all(&self.pool).await?;

        let count: i64 = count_builder
            .build_query_scalar()
            .fetch_one(&self.pool)
            .await?;

        tracing::info!(user_id = %user_id, total = count, "Trade history fetched");
        Ok((trades, count))
    }
}
