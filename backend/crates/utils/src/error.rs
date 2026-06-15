use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

use loyalty_engine::error::EngineError;

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    Internal(String),
}

impl AppError {
    fn parts(&self) -> (StatusCode, &str) {
        match self {
            AppError::NotFound(m) => (StatusCode::NOT_FOUND, m.as_str()),
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

impl From<EngineError> for AppError {
    fn from(e: EngineError) -> Self {
        match e {
            EngineError::NotFound(m) => AppError::NotFound(m),
            EngineError::Db(m) => AppError::Internal(m),
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
