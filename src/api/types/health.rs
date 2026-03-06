#[derive(serde::Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}
