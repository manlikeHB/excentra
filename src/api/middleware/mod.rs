use std::sync::Arc;

use crate::{
    api::types::AppState,
    auth::{Claims, verify_token},
};
use axum::response::IntoResponse;
use axum::{Json, extract::FromRequestParts, http::StatusCode};

pub struct AuthUser(pub Claims);

impl FromRequestParts<Arc<AppState>> for AuthUser {
    type Rejection = AuthRejection;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let header_value = match parts.headers.get("authorization") {
            Some(token) => token,
            None => return Err(AuthRejection::NoAuthorizationHeader),
        };

        let bearer_token = match header_value.to_str() {
            Ok(bearer_token) => bearer_token,
            Err(_) => return Err(AuthRejection::InvalidHeaderValue),
        };

        let token = bearer_token
            .strip_prefix("Bearer ")
            .ok_or(AuthRejection::InvalidBearerToken)?;

        // if it fails
        // - token expired or
        // - token is invalid, hence not logged in
        Ok(AuthUser(verify_token(token, &state.jwt_secret)?))
    }
}

impl IntoResponse for AuthRejection {
    fn into_response(self) -> axum::response::Response {
        let msg = match self {
            AuthRejection::FailedToAuthorizeUser => "User not logged in",
            AuthRejection::InvalidBearerToken => "Invalid Bearer token",
            AuthRejection::InvalidHeaderValue => "Invalid Header value",
            AuthRejection::NoAuthorizationHeader => "No authorization header",
        };

        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response()
    }
}

pub enum AuthRejection {
    NoAuthorizationHeader,
    InvalidHeaderValue,
    InvalidBearerToken,
    FailedToAuthorizeUser,
}

impl From<jsonwebtoken::errors::Error> for AuthRejection {
    fn from(_: jsonwebtoken::errors::Error) -> Self {
        AuthRejection::FailedToAuthorizeUser
    }
}
