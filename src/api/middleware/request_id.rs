use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};
use tracing::Instrument;
use uuid::Uuid;

// The tracing span key that will appear in every log line
const REQUEST_ID_HEADER: &str = "x-request-id";

pub async fn request_id_middleware(request: Request, next: Next) -> Response {
    // Check if caller already provided an x-request-id header
    let id = request
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Create a tracing span with the request_id attached:
    let span = tracing::info_span!("req", request_id = %id);

    // Run the rest of the request pipeline inside that span:
    let mut response = next.run(request).instrument(span).await;

    // Add the request_id as a response header too
    response.headers_mut().insert(
        REQUEST_ID_HEADER,
        HeaderValue::from_str(&id).expect("UUID is always a valid header value"),
    );

    response
}
