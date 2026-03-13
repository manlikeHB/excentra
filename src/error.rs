use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

use crate::types::asset::AssetSymbolError;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Order not found")]
    OrderNotFound,
    #[error("Limit order must have a price")]
    MissingPrice,
    #[error("Pair not found")]
    PairNotFound,
    #[error("Order found in index but not in book")]
    InconsistentState,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Unprocessable: {0}")]
    Unprocessable(String),
    #[error("Forbidden: {0}")]
    Forbidden(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status_code, msg) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            AppError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::Unprocessable(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
        };

        (status_code, Json(serde_json::json!({"error": msg}))).into_response()
    }
}

impl From<bcrypt::BcryptError> for AppError {
    fn from(_e: bcrypt::BcryptError) -> Self {
        AppError::InternalError("Password hashing failed".to_string())
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                AppError::Conflict("Resource already exist".to_string())
            }
            _ => {
                eprintln!("Database error: {:?}", e);
                AppError::InternalError("Database error".to_string())
            }
        }
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(e: validator::ValidationErrors) -> Self {
        let messages: Vec<String> = e
            .field_errors()
            .into_iter()
            .map(|(field, errors)| {
                let msgs: Vec<&str> = errors
                    .iter()
                    .filter_map(|e| e.message.as_ref().map(|m| m.as_ref()))
                    .collect();
                if msgs.is_empty() {
                    format!("Invalid {}", field)
                } else {
                    msgs.join(", ")
                }
            })
            .collect();

        AppError::BadRequest(messages.join("; "))
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(_: jsonwebtoken::errors::Error) -> Self {
        AppError::InternalError("jwt error".to_string())
    }
}

impl From<EngineError> for AppError {
    fn from(value: EngineError) -> Self {
        match value {
            EngineError::MissingPrice => {
                AppError::BadRequest("Limit Order should have price".to_string())
            }
            EngineError::OrderNotFound => AppError::NotFound("Order not found".to_string()),
            EngineError::PairNotFound => AppError::NotFound("Asset pair not found".to_string()),
            EngineError::InconsistentState => {
                AppError::InternalError("Order found in index but not in book".to_string())
            }
        }
    }
}

impl From<AssetSymbolError> for AppError {
    fn from(value: AssetSymbolError) -> Self {
        match value {
            AssetSymbolError::InvalidSymbol => {
                AppError::BadRequest("Invalid asset symbol, expected e.g `BTC/USDT`".to_string())
            }
            AssetSymbolError::InvalidSymbolFormReqPath => {
                AppError::BadRequest("Invalid asset symbol, expected e.g `BTC-USDT`".to_string())
            }
            AssetSymbolError::MarketNotSupported(m) => {
                AppError::BadRequest(format!("{} market not supported", m))
            }
        }
    }
}
