use std::sync::Arc;

use crate::{
    api::types::AppState,
    auth::{Claims, verify_token},
    db::models::user::UserRole,
};
use axum::http::request::Parts;
use axum::response::IntoResponse;
use axum::{Json, extract::FromRequestParts, http::StatusCode};

pub struct AuthUser(pub Claims);
pub struct AdminUser(pub Claims);

impl FromRequestParts<Arc<AppState>> for AuthUser {
    type Rejection = AuthRejection;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let claims = extract_claims(parts, &state.jwt_secret)?;
        Ok(AuthUser(claims))
    }
}

impl FromRequestParts<Arc<AppState>> for AdminUser {
    type Rejection = AuthRejection;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let claims = extract_claims(parts, &state.jwt_secret)?;

        // verify it's admin
        if claims.role() != UserRole::Admin {
            return Err(AuthRejection::InsufficientPermissions);
        }

        Ok(AdminUser(claims))
    }
}

fn extract_claims(parts: &mut Parts, jwt_secret: &str) -> Result<Claims, AuthRejection> {
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
    Ok(verify_token(token, jwt_secret)?)
}

impl IntoResponse for AuthRejection {
    fn into_response(self) -> axum::response::Response {
        let msg = match self {
            AuthRejection::FailedToAuthorizeUser => "User not logged in",
            AuthRejection::InvalidBearerToken => "Invalid Bearer token",
            AuthRejection::InvalidHeaderValue => "Invalid Header value",
            AuthRejection::NoAuthorizationHeader => "No authorization header",
            AuthRejection::InsufficientPermissions => {
                "You do not have permission to perform this action."
            }
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
    InsufficientPermissions,
}

impl From<jsonwebtoken::errors::Error> for AuthRejection {
    fn from(_: jsonwebtoken::errors::Error) -> Self {
        AuthRejection::FailedToAuthorizeUser
    }
}
