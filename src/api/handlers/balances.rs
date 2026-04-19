use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::{
    api::{
        middleware::{auth::AuthUser, rate_limit::policies},
        types::{
            AppState,
            balances::{BalanceRequest, BalanceResponse},
        },
    },
    error::AppError,
};

#[utoipa::path(
    post,
    path = "/api/v1/balances/deposit",
    tag = "Balances",
    request_body = BalanceRequest,
    responses(
        (status = 200, description = "Deposit successful", body = BalanceResponse),
        (status = 400, description = "Invalid amount"),
        (status = 401, description = "Not authenticated"),
        (status = 422, description = "Asset not supported"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn deposit(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<BalanceRequest>,
) -> Result<(StatusCode, Json<BalanceResponse>), AppError> {
    let user_id = auth.0.user_id();
    state
        .rate_limiter
        .check(user_id.to_string(), &policies::DEPOSIT)?;
    body.validate()?;
    let asset = &body.asset.trim().to_uppercase();

    let bal = state
        .balance_service
        .deposit(user_id, body.amount, asset)
        .await?;

    Ok((StatusCode::OK, Json(bal.into())))
}

#[utoipa::path(
    get,
    path = "/api/v1/balances",
    tag = "Balances",
    responses(
        (status = 200, description = "Balances fetched successfully", body = Vec<BalanceResponse>),
        (status = 401, description = "Not authenticated"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
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

#[utoipa::path(
    post,
    path = "/api/v1/balances/withdraw",
    tag = "Balances",
    request_body = BalanceRequest,
    responses(
        (status = 200, description = "Withdrawal successful", body = BalanceResponse),
        (status = 400, description = "Invalid amount"),
        (status = 401, description = "Not authenticated"),
        (status = 422, description = "Asset not supported or insufficient balance"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn withdraw(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<BalanceRequest>,
) -> Result<(StatusCode, Json<BalanceResponse>), AppError> {
    let user_id = auth.0.user_id();
    state
        .rate_limiter
        .check(user_id.to_string(), &policies::WITHDRAW)?;

    body.validate()?;
    let asset = &body.asset.trim().to_uppercase();

    let bal = state
        .balance_service
        .withdraw(user_id, body.amount, asset)
        .await?;

    Ok((StatusCode::OK, Json(bal.into())))
}

#[utoipa::path(
    get,
    path = "/api/v1/balances/{asset}",
    tag = "Balances",
    params(("asset" = String, Path, description = "Asset symbol e.g BTC")),
    responses(
        (status = 200, description = "Balance fetched successfully", body = BalanceResponse),
        (status = 401, description = "Not authenticated"),
        (status = 422, description = "Asset not supported"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
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
