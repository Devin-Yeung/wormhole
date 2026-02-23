use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use wormhole_shortener::ShortenerError;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    InvalidRequest(String),
    InvalidUrl(String),
    InvalidShortCode(String),
    AliasConflict(String),
    StorageUnavailable(String),
    StorageTimeout(String),
    Internal(String),
}

impl AppError {
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::InvalidRequest(message.into())
    }

    fn status_and_code(&self) -> (StatusCode, &'static str) {
        match self {
            Self::InvalidRequest(_) => (StatusCode::BAD_REQUEST, "invalid_request"),
            Self::InvalidUrl(_) => (StatusCode::BAD_REQUEST, "invalid_url"),
            Self::InvalidShortCode(_) => (StatusCode::BAD_REQUEST, "invalid_short_code"),
            Self::AliasConflict(_) => (StatusCode::CONFLICT, "alias_conflict"),
            Self::StorageUnavailable(_) => (StatusCode::SERVICE_UNAVAILABLE, "storage_unavailable"),
            Self::StorageTimeout(_) => (StatusCode::GATEWAY_TIMEOUT, "storage_timeout"),
            Self::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
        }
    }

    fn message(self) -> String {
        match self {
            Self::InvalidRequest(message)
            | Self::InvalidUrl(message)
            | Self::InvalidShortCode(message)
            | Self::StorageUnavailable(message)
            | Self::StorageTimeout(message)
            | Self::Internal(message) => message,
            Self::AliasConflict(_) => "short code already exists".to_string(),
        }
    }
}

impl From<ShortenerError> for AppError {
    fn from(error: ShortenerError) -> Self {
        match error {
            ShortenerError::AliasConflict(code) => Self::AliasConflict(code),
            ShortenerError::InvalidUrl(message) => Self::InvalidUrl(message),
            ShortenerError::InvalidShortCode(message) => Self::InvalidShortCode(message),
            ShortenerError::Storage(message) => {
                if message.starts_with("storage backend unavailable:") {
                    Self::StorageUnavailable(message)
                } else if message.starts_with("storage operation timed out:") {
                    Self::StorageTimeout(message)
                } else {
                    Self::Internal(message)
                }
            }
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = self.status_and_code();

        let resp = serde_json::json!({
            "error": {
                "code": code,
                "message": self.message(),
            }
        });

        (status, Json(resp)).into_response()
    }
}
