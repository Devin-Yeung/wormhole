use thiserror::Error;
use wormhole_core::CoreError;

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

impl From<CoreError> for ShortenerError {
    fn from(value: CoreError) -> Self {
        match value {
            CoreError::InvalidShortCode(message) => Self::InvalidShortCode(message),
        }
    }
}
