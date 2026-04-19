use axum::http::HeaderMap;
use std::net::SocketAddr;

pub fn extract_ip(headers: &HeaderMap, addr: SocketAddr) -> String {
    headers
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|ip| ip.trim().to_string())
        .unwrap_or_else(|| addr.ip().to_string())
}
