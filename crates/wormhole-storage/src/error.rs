use thiserror::Error;
use wormhole_core::CacheError;

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
