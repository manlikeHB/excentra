use sqlx::{PgPool, QueryBuilder};

use crate::{
    constants,
    db::{models::order::DBOrderStatus, queries as db_queries},
    error::AppError,
    types::asset_symbol::AssetSymbol,
};

pub async fn apply_filters<'q>(
    pool: &PgPool,
    builder: &mut QueryBuilder<'q, sqlx::Postgres>,
    status: Option<DBOrderStatus>,
    pair: Option<&str>,
) -> Result<(), AppError> {
    if let Some(s) = status {
        builder.push(" AND status = ");
        builder.push_bind(s.clone());
    }

    if let Some(p) = pair {
        let symbol = AssetSymbol::from_path(&p)?;
        if let Some(pair_id) = db_queries::find_by_symbol(pool, symbol.as_str()).await? {
            builder.push(" AND pair_id = ");
            builder.push_bind(pair_id.id);
        } else {
            return Err(AppError::Unprocessable(format!(
                "Invalid pair symbol: {}",
                symbol.as_str()
            )));
        }
    }

    Ok(())
}

pub fn apply_pagination<'q>(
    builder: &mut QueryBuilder<'q, sqlx::Postgres>,
    page: Option<u64>,
    limit: Option<u64>,
) {
    match (page, limit) {
        (Some(p), Some(l)) => {
            let offset = (p - 1) * l;
            builder.push(" ORDER BY o.created_at DESC LIMIT ");
            builder.push_bind(l as i64);
            builder.push(" OFFSET ");
            builder.push_bind(offset as i64);
        }
        (Some(p), None) => {
            let offset = (p - 1) * constants::DEFAULT_PAGE_SIZE;
            builder.push(" ORDER BY o.created_at DESC LIMIT ");
            builder.push_bind(constants::DEFAULT_PAGE_SIZE as i64);
            builder.push(" OFFSET ");
            builder.push_bind(offset as i64);
        }
        (None, Some(l)) => {
            builder.push(" ORDER BY o.created_at DESC LIMIT ");
            builder.push_bind(l as i64);
        }
        (None, None) => {
            builder.push(" ORDER BY o.created_at DESC LIMIT ");
            builder.push_bind(constants::DEFAULT_PAGE_SIZE as i64);
        }
    }
}
