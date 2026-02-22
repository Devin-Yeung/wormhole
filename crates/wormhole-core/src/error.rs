use thiserror::Error;

/// Errors related to the core functionality of the URL shortener service.
pub type Result<T> = std::result::Result<T, CoreError>;

#[derive(Debug, Clone, Error)]
pub enum CoreError {
    #[error("invalid short code: {0}")]
    InvalidShortCode(String),
}
