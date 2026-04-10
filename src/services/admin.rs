use sqlx::{PgPool, QueryBuilder};
use uuid::Uuid;

use crate::{
    api::types::admin::{AdminStats, PairVolume, UserSummary},
    db::{
        models::user::{User, UserRole},
        queries as db_queries,
    },
    error::AppError,
    utils::{
        query_builder::{self, QueryOrder},
        ticker::get_ticker_helper,
    },
};

pub struct AdminService {
    pub pool: PgPool,
}

impl AdminService {
    pub fn new(pool: PgPool) -> Self {
        AdminService { pool }
    }

    pub async fn get_all_users_summary(
        &self,
        page: Option<u64>,
        limit: Option<u64>,
        order: Option<QueryOrder>,
    ) -> Result<(Vec<UserSummary>, i64), AppError> {
        let mut user_summary_builder = QueryBuilder::new(
            "
            SELECT 
                u.id,
                u.email,
                u.role,
                u.username,
                u.created_at,
                u.updated_at,
                u.is_suspended,
                COALESCE(
                    JSON_AGG(
                        JSON_BUILD_OBJECT(
                            'asset', b.asset,
                            'available', b.available,
                            'held', b.held
                        )
                    ) FILTER (WHERE b.asset IS NOT NULL),
                    '[]'
                ) AS balances
            FROM users u
            LEFT JOIN balances b ON b.user_id = u.id
            GROUP BY u.id 
        ",
        );

        query_builder::apply_pagination(&mut user_summary_builder, page, limit, "u", order);
        let summaries: Vec<UserSummary> = user_summary_builder
            .build_query_as()
            .fetch_all(&self.pool)
            .await?;
        let count = db_queries::count_users(&self.pool).await?;

        tracing::info!(count = count, "Users summaries Fetched");

        Ok((summaries, count))
    }

    pub async fn suspend_user(&self, user_id: Uuid, suspended: bool) -> Result<User, AppError> {
        let user = match db_queries::suspend_user(&self.pool, user_id, suspended).await? {
            Some(u) => u,
            None => return Err(AppError::NotFound(format!("User `{}` not found", user_id))),
        };

        tracing::info!(user_id = %user_id, suspended = suspended, "User suspension updated");

        Ok(user)
    }

    pub async fn update_role(&self, user_id: Uuid, role: UserRole) -> Result<User, AppError> {
        let user = match db_queries::update_role(&self.pool, user_id, role).await? {
            Some(u) => u,
            None => return Err(AppError::NotFound(format!("User `{}` not found", user_id))),
        };

        tracing::info!(user_id = %user_id, role = ?role, "User role updated");

        Ok(user)
    }

    pub async fn get_stats(
        &self,
        active_ws_connections: u64,
        orders_processed: u64,
        uptime_seconds: u64,
    ) -> Result<AdminStats, AppError> {
        let total_users = db_queries::count_users(&self.pool).await?;
        let total_trades = db_queries::count_trades(&self.pool).await?;
        let mut volume_24h = vec![];

        let trade_stats = db_queries::get_all_trade_stats(&self.pool).await?;

        for trade_stat in trade_stats {
            match get_ticker_helper(&trade_stat) {
                Some(t) => {
                    let pair_volume = PairVolume {
                        symbol: t.symbol,
                        volume: t.volume_24h,
                    };
                    volume_24h.push(pair_volume);
                }
                None => tracing::warn!(pair = %trade_stat.symbol, "No trades found for"),
            }
        }

        let admin_stat = AdminStats {
            total_users,
            total_trades,
            active_ws_connections,
            orders_processed,
            uptime_seconds,
            volume_24h,
        };

        Ok(admin_stat)
    }
}
