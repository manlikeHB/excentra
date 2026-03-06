use crate::api::types::health::HealthResponse;
use axum::{Json, http::StatusCode};

pub async fn health() -> (StatusCode, Json<HealthResponse>) {
    (StatusCode::OK, Json(HealthResponse { status: "Ok" }))
}
