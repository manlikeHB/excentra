use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};

use crate::{
    api::{
        middleware::AuthUser,
        types::{
            AppState,
            balances::{BalanceRequest, BalanceResponse},
        },
    },
    db::queries as db_queries,
    error::AppError,
};

// TODO: Impl real deposit from supported blockchain
pub async fn deposit(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<BalanceRequest>,
) -> Result<(StatusCode, Json<BalanceResponse>), AppError> {
    let user_id = auth.0.user_id();
    let asset = &body.asset.to_uppercase();

    // verify asset is supported
    if !db_queries::is_valid_asset(&state.pool, asset).await? {
        return Err(AppError::BadRequest(format!(
            "{} is not supported",
            &body.asset
        )));
    }

    let bal = db_queries::deposit(&state.pool, user_id, asset, body.amount).await?;

    tracing::info!(user_id = %user_id, asset = %asset, amount = %body.amount, "Deposit credited");

    Ok((StatusCode::OK, Json(bal.into())))
}

pub async fn get_balances(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<Vec<BalanceResponse>>), AppError> {
    let user_id = auth.0.user_id();
    let mut bal_vec = Vec::new();

    let balances = db_queries::get_balances(&state.pool, user_id).await?;

    for bal in balances {
        bal_vec.push(bal.into());
    }

    Ok((StatusCode::OK, Json(bal_vec)))
}
