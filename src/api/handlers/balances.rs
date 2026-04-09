use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::{
    api::{
        middleware::AuthUser,
        types::{
            AppState,
            balances::{BalanceRequest, BalanceResponse},
        },
    },
    error::AppError,
};

pub async fn deposit(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<BalanceRequest>,
) -> Result<(StatusCode, Json<BalanceResponse>), AppError> {
    body.validate()?;
    let user_id = auth.0.user_id();
    let asset = &body.asset.trim().to_uppercase();

    let bal = state
        .balance_service
        .deposit(user_id, body.amount, asset)
        .await?;

    Ok((StatusCode::OK, Json(bal.into())))
}

pub async fn get_balances(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<Vec<BalanceResponse>>), AppError> {
    let user_id = auth.0.user_id();

    let balances = state
        .balance_service
        .get_balances(user_id)
        .await?
        .into_iter()
        .map(|b| b.into())
        .collect();

    Ok((StatusCode::OK, Json(balances)))
}

pub async fn withdraw(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<BalanceRequest>,
) -> Result<(StatusCode, Json<BalanceResponse>), AppError> {
    body.validate()?;
    let user_id = auth.0.user_id();
    let asset = &body.asset.trim().to_uppercase();

    let bal = state
        .balance_service
        .withdraw(user_id, body.amount, asset)
        .await?;

    Ok((StatusCode::OK, Json(bal.into())))
}

pub async fn get_balance(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(asset): Path<String>,
) -> Result<(StatusCode, Json<BalanceResponse>), AppError> {
    let user_id = auth.0.user_id();
    let asset = &asset.trim().to_uppercase();

    let balance = state.balance_service.get_balance(user_id, asset).await?;

    Ok((StatusCode::OK, Json(balance.into())))
}
