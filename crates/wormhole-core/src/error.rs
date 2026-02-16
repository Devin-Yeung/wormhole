use thiserror::Error;

/// Type alias for the result type used in the URL shortener service.
pub type Result<T> = std::result::Result<T, Error>;

// Centralized error type for the URL shortener service, encompassing all possible error cases
#[derive(Debug, Error)]
pub enum Error {
    #[error("cache error: {0}")]
    Cache(#[from] CacheError),
    #[error("storage error: {0}")]
    Storage(#[from] StorageError),
    #[error("shortener error: {0}")]
    Shortener(#[from] ShortenerError),
}

#[derive(Debug, Error)]
pub enum CacheError {
    #[error(transparent)]
    Other(Box<dyn std::error::Error + Send + Sync>),
}

impl From<Box<dyn std::error::Error + Send + Sync>> for CacheError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::Other(err)
    }
}

#[derive(Debug, Error)]
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
    #[error(transparent)]
    Other(Box<dyn std::error::Error + Send + Sync>),
}

impl From<Box<dyn std::error::Error + Send + Sync>> for StorageError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::Other(err)
    }
}

#[derive(Debug, Error)]
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
