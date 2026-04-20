use sqlx::{PgPool, QueryBuilder};

use crate::{
    constants,
    db::{models::order::DBOrderStatus, queries as db_queries},
    error::AppError,
    types::asset_symbol::AssetSymbol,
};

#[derive(Debug, serde::Deserialize, Clone, Copy)]
// #[serde(rename_all = "lowercase")]
pub enum QueryOrder {
    ASC,
    DESC,
}

pub fn apply_status_filter<'q>(
    builder: &mut QueryBuilder<'q, sqlx::Postgres>,
    status: &Option<Vec<DBOrderStatus>>,
) {
    if let Some(status_vec) = status {
        builder.push(" AND status IN (");

        let mut separated = builder.separated(", ");
        for status in status_vec {
            separated.push_bind(*status);
        }
        builder.push(")");
    }
}

pub async fn apply_pair_filter<'q>(
    pool: &PgPool,
    builder: &mut QueryBuilder<'q, sqlx::Postgres>,
    pair: Option<&str>,
    table: &str,
) -> Result<(), AppError> {
    if let Some(p) = pair {
        let symbol = AssetSymbol::from_path(&p)?;
        if let Some(pair_id) = db_queries::find_by_symbol(pool, symbol.as_str()).await? {
            builder.push(format!(" AND {}.pair_id = ", table));
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
    table: &str,
    order: Option<QueryOrder>,
) {
    let order = match order {
        Some(QueryOrder::ASC) => "ASC",
        Some(QueryOrder::DESC) | None => "DESC",
    };

    match (page, limit) {
        (Some(p), Some(l)) => {
            let offset = (p - 1) * l;
            builder.push(format!(" ORDER BY {}.created_at {} LIMIT ", table, order));
            builder.push_bind(l as i64);
            builder.push(" OFFSET ");
            builder.push_bind(offset as i64);
        }
        (Some(p), None) => {
            let offset = (p - 1) * constants::DEFAULT_PAGE_SIZE;
            builder.push(format!(" ORDER BY {}.created_at {} LIMIT ", table, order));
            builder.push_bind(constants::DEFAULT_PAGE_SIZE as i64);
            builder.push(" OFFSET ");
            builder.push_bind(offset as i64);
        }
        (None, Some(l)) => {
            builder.push(format!(" ORDER BY {}.created_at {} LIMIT ", table, order));
            builder.push_bind(l as i64);
        }
        (None, None) => {
            builder.push(format!(" ORDER BY {}.created_at {} LIMIT ", table, order));
            builder.push_bind(constants::DEFAULT_PAGE_SIZE as i64);
        }
    }
}
