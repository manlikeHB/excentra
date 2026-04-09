use crate::api::types::auth::{LoginRequest, LoginResponse};
use crate::{
    api::types::{AppState, auth::RegisterRequest},
    error::AppError,
};
use axum::{Json, extract::State, http::StatusCode};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use std::sync::Arc;
use validator::Validate;

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
