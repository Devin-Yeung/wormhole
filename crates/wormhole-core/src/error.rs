use thiserror::Error;

/// Type alias for the result type used in the URL shortener service.
pub type Result<T> = std::result::Result<T, Error>;

/// Custom error type for the URL shortener service.
#[derive(Debug, Error)]
pub enum Error {
    #[error("storage error: {0}")]
    Storage(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("alias already exists: {0}")]
    AliasConflict(String),
    #[error("invalid url: {0}")]
    InvalidUrl(String),
    #[error("invalid short code: {0}")]
    InvalidShortCode(String),
}
