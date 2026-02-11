use thiserror::Error;

/// Type alias for the result type used in the URL shortener service.
pub type Result<T> = std::result::Result<T, Error>;

/// Custom error type for the URL shortener service.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Database error: {0}")]
    StorageError(String),
    #[error("Alias already exists")]
    AliasConflict,
}
