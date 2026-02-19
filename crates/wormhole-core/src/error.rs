use thiserror::Error;

/// Errors related to the core functionality of the URL shortener service.
pub type Result<T> = std::result::Result<T, CoreError>;

#[derive(Debug, Clone, Error)]
pub enum CacheError {
    #[error("cache backend unavailable: {0}")]
    Unavailable(String),
    #[error("cache operation timed out: {0}")]
    Timeout(String),
    #[error("cache serialization failed: {0}")]
    Serialization(String),
    #[error("cache value is invalid: {0}")]
    InvalidData(String),
    #[error("cache initialization failed: {0}")]
    Initialization(String),
    #[error("cache operation failed: {0}")]
    Operation(String),
}

#[derive(Debug, Clone, Error)]
pub enum CoreError {
    #[error("invalid short code: {0}")]
    InvalidShortCode(String),
}
