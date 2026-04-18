use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};

use crate::{
    api::{
        middleware::AdminUser,
        types::{AppState, asset::AddAssetRequest},
    },
    db::models::assets::Asset,
    error::AppError,
};

#[utoipa::path(
    post,
    path = "/api/v1/assets",
    tag = "Assets",
    request_body = AddAssetRequest,
    responses(
        (status = 201, description = "Asset created successfully", body = Asset),
        (status = 400, description = "Invalid asset data"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Admin access required"),
        (status = 409, description = "Asset already exists"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn add_asset(
    _auth: AdminUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<AddAssetRequest>,
) -> Result<(StatusCode, Json<Asset>), AppError> {
    body.validate()?;
    let body = body.normalize()?;

    let asset = state
        .asset_service
        .add_asset(&body.symbol, &body.name, body.decimals)
        .await?;
    Ok((StatusCode::CREATED, Json(asset)))
}

#[utoipa::path(
    get,
    path = "/api/v1/assets",
    tag = "Assets",
    responses(
        (status = 200, description = "Assets fetched successfully", body = Vec<Asset>),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn get_all_assets(
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<Vec<Asset>>), AppError> {
    let assets = state.asset_service.get_all_asset().await?;

    Ok((StatusCode::OK, Json(assets)))
}
