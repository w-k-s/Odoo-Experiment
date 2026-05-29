use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

/// Application-wide error type. Hand-rolled (no thiserror) so the dependency
/// surface stays small.
#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    BadRequest(String),
    Unauthorized(String),
    /// Anything we don't want to leak details about (DB, pool, interact join).
    Internal(String),
}

impl AppError {
    fn parts(&self) -> (StatusCode, &str) {
        match self {
            AppError::NotFound(m) => (StatusCode::NOT_FOUND, m.as_str()),
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, m.as_str()),
            AppError::Unauthorized(m) => (StatusCode::UNAUTHORIZED, m.as_str()),
            AppError::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, m.as_str()),
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (_, msg) = self.parts();
        write!(f, "{msg}")
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = self.parts();
        if status == StatusCode::INTERNAL_SERVER_ERROR {
            tracing::error!("internal error: {message}");
        }
        (status, Json(json!({ "error": message }))).into_response()
    }
}

// ---- Conversions from lower-level errors into a 500 (details logged) ----

impl From<diesel::result::Error> for AppError {
    fn from(e: diesel::result::Error) -> Self {
        match e {
            diesel::result::Error::NotFound => AppError::NotFound("not found".into()),
            other => AppError::Internal(format!("db error: {other}")),
        }
    }
}

impl From<deadpool_diesel::PoolError> for AppError {
    fn from(e: deadpool_diesel::PoolError) -> Self {
        AppError::Internal(format!("pool error: {e}"))
    }
}

impl From<deadpool_diesel::InteractError> for AppError {
    fn from(e: deadpool_diesel::InteractError) -> Self {
        AppError::Internal(format!("interact error: {e}"))
    }
}

pub type AppResult<T> = Result<T, AppError>;
