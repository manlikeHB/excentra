#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct HealthResponse {
    pub status: &'static str,
    pub db: bool,
    pub uptime_seconds: u64,
    pub ws_connections: u64,
    pub orders_processed: u64,
}
