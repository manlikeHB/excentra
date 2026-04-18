use crate::api::types::auth::{LoginRequest, LoginResponse};
use crate::{
    api::types::{AppState, auth::RegisterRequest},
    error::AppError,
};
use axum::{Json, extract::State, http::StatusCode};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use std::sync::Arc;
use validator::Validate;

#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    tag = "Auth",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = LoginResponse),
        (status = 400, description = "Invalid email or password format"),
        (status = 409, description = "Email already registered"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn register_user(
    jar: CookieJar,
    State(state): State<Arc<AppState>>,
    Json(body): Json<RegisterRequest>,
) -> Result<(CookieJar, (StatusCode, Json<LoginResponse>)), AppError> {
    // validate email
    body.validate()?;
    let (access_token, refresh_token) = state
        .auth_service
        .register_user(&body.email, &body.password)
        .await?;

    Ok((
        set_refresh_cookie(jar, &state.base_url, refresh_token),
        (StatusCode::CREATED, Json(LoginResponse { access_token })),
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    tag = "Auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 400, description = "Invalid request body"),
        (status = 401, description = "Invalid email or password"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn login_user(
    jar: CookieJar,
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoginRequest>,
) -> Result<(CookieJar, (StatusCode, Json<LoginResponse>)), AppError> {
    body.validate()?;

    let (access_token, refresh_token) = state
        .auth_service
        .login(&body.email, &body.password)
        .await?;

    Ok((
        set_refresh_cookie(jar, &state.base_url, refresh_token),
        (StatusCode::OK, Json(LoginResponse { access_token })),
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    tag = "Auth",
    responses(
        (status = 200, description = "Token refreshed successfully", body = LoginResponse),
        (status = 401, description = "Missing or expired refresh token"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn refresh_token(
    jar: CookieJar,
    State(state): State<Arc<AppState>>,
) -> Result<(CookieJar, (StatusCode, Json<LoginResponse>)), AppError> {
    let (jar, refresh_token) = remove_refresh_token(jar)?;

    let (access_token, refresh_token) = state.auth_service.refresh_token(&refresh_token).await?;

    Ok((
        set_refresh_cookie(jar, &state.base_url, refresh_token),
        (StatusCode::OK, Json(LoginResponse { access_token })),
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    tag = "Auth",
    responses(
        (status = 204, description = "Logged out successfully"),
        (status = 401, description = "Missing or expired refresh token"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn logout(
    jar: CookieJar,
    State(state): State<Arc<AppState>>,
) -> Result<(CookieJar, StatusCode), AppError> {
    let (jar, refresh_token) = remove_refresh_token(jar)?;

    state.auth_service.logout(&refresh_token).await?;

    Ok((jar, StatusCode::NO_CONTENT))
}

fn set_refresh_cookie(jar: CookieJar, base_url: &str, refresh_token: String) -> CookieJar {
    let cookie = Cookie::build(("refresh_token", refresh_token))
        .path(format!("{}/auth", base_url))
        .http_only(true)
        .secure(false); // TODO: set true in prod

    jar.add(cookie)
}

fn remove_refresh_token(jar: CookieJar) -> Result<(CookieJar, String), AppError> {
    let refresh_token = jar
        .get("refresh_token")
        .map(|t| t.value().to_owned())
        .ok_or(AppError::Unauthorized("No refresh token".to_string()))?;

    Ok((jar.remove("refresh_token"), refresh_token))
}
