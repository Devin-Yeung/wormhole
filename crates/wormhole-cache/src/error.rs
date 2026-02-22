use thiserror::Error;

/// Type alias for cache results.
pub type Result<T> = std::result::Result<T, CacheError>;

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
