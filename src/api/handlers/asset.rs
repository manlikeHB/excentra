use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};

use crate::{
    api::{
        middleware::AuthUser,
        types::{AppState, asset::AddAssetRequest},
    },
    db::models::assets::Asset,
    error::AppError,
};

pub async fn add_asset(
    _auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<AddAssetRequest>,
) -> Result<(StatusCode, Json<Asset>), AppError> {
    body.validate()?;
    let body = body.normalize()?;

    let asset = state
        .asset_service
        .add_asset(&body.symbol, &body.name, body.decimals)
        .await?;
    Ok((StatusCode::OK, Json(asset)))
}

pub async fn get_all_assets(
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<Vec<Asset>>), AppError> {
    let assets = state.asset_service.get_all_asset().await?;

    Ok((StatusCode::OK, Json(assets)))
}
