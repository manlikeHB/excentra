use crate::api::types::{AppState, health::HealthResponse};
use axum::{Json, extract::State, http::StatusCode};
use std::sync::Arc;
use std::sync::atomic::Ordering;

#[utoipa::path(
    get,
    path = "/health",
    tag = "system",
    responses(
        (status = 200, description = "System is healthy", body = HealthResponse),
        (status = 503, description = "System degraded"),
    )
)]
pub async fn health(State(state): State<Arc<AppState>>) -> (StatusCode, Json<HealthResponse>) {
    let db = sqlx::query("SELECT 1").execute(&state.pool).await.is_ok();

    let status_code = if db {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status_code,
        Json(HealthResponse {
            status: if db { "ok" } else { "degraded" },
            db,
            uptime_seconds: state.started_at.elapsed().as_secs(),
            ws_connections: state.ws_connections.load(Ordering::Relaxed),
            orders_processed: state.order_service.orders_processed(),
        }),
    )
}
