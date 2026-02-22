use std::sync::Arc;
use thiserror::Error;
use tonic::{Code, Status};
use wormhole_cache::CacheError;

/// Result type for repository operations.
pub type Result<T> = std::result::Result<T, StorageError>;

#[derive(Debug, Clone, Error)]
pub enum StorageError {
    #[error("alias already exists: {0}")]
    Conflict(String),
    #[error("storage backend unavailable: {0}")]
    Unavailable(String),
    #[error("storage operation timed out: {0}")]
    Timeout(String),
    #[error("storage query failed: {0}")]
    Query(String),
    #[error("stored data is invalid: {0}")]
    InvalidData(String),
    #[error("cache error: {0}")]
    Cache(#[from] CacheError),
    #[error("storage operation failed: {0}")]
    Operation(String),
}

impl From<StorageError> for Status {
    fn from(error: StorageError) -> Self {
        let (code, message) = match &error {
            StorageError::Unavailable(_) => (Code::Unavailable, "storage backend unavailable"),
            StorageError::Timeout(_) => (Code::DeadlineExceeded, "storage operation timed out"),
            StorageError::Conflict(_) => (Code::AlreadyExists, "short code already exists"),
            StorageError::InvalidData(_)
            | StorageError::Query(_)
            | StorageError::Cache(_)
            | StorageError::Operation(_) => (Code::Internal, "storage operation failed"),
        };

        let mut status = Status::new(code, message);
        status.set_source(Arc::new(error));
        status
    }
}

#[cfg(test)]
mod tests {
    use super::StorageError;
    use tonic::{Code, Status};

    fn assert_status(error: StorageError, expected_code: Code, expected_message: &str) {
        let status: Status = error.into();
        assert_eq!(status.code(), expected_code);
        assert_eq!(status.message(), expected_message);
    }

    #[test]
    fn storage_error_unavailable_maps_to_unavailable() {
        assert_status(
            StorageError::Unavailable("db down".to_string()),
            Code::Unavailable,
            "storage backend unavailable",
        );
    }

    #[test]
    fn storage_error_timeout_maps_to_deadline_exceeded() {
        assert_status(
            StorageError::Timeout("slow query".to_string()),
            Code::DeadlineExceeded,
            "storage operation timed out",
        );
    }

    #[test]
    fn storage_error_conflict_maps_to_already_exists() {
        assert_status(
            StorageError::Conflict("abc123".to_string()),
            Code::AlreadyExists,
            "short code already exists",
        );
    }

    #[test]
    fn storage_error_internal_group_maps_to_internal() {
        assert_status(
            StorageError::InvalidData("bad record".to_string()),
            Code::Internal,
            "storage operation failed",
        );
        assert_status(
            StorageError::Query("syntax error".to_string()),
            Code::Internal,
            "storage operation failed",
        );
        assert_status(
            StorageError::Operation("write failed".to_string()),
            Code::Internal,
            "storage operation failed",
        );
    }
}
