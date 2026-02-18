use thiserror::Error;

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
pub enum ShortenerError {
    #[error("alias already exists: {0}")]
    AliasConflict(String),
    #[error("invalid url: {0}")]
    InvalidUrl(String),
    #[error("invalid short code: {0}")]
    InvalidShortCode(String),
    #[error("storage error: {0}")]
    Storage(String),
}
