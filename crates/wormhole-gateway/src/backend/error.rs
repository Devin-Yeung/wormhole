use wormhole_redirector::RedirectorError;
use wormhole_shortener::ShortenerError;

#[derive(Debug)]
pub enum BackendError {
    InvalidUrl(String),
    InvalidShortCode(String),
    NotFound,
    AliasConflict(String),
    StorageUnavailable(String),
    StorageTimeout(String),
    Internal(String),
}

pub type Result<T> = std::result::Result<T, BackendError>;

impl From<ShortenerError> for BackendError {
    fn from(error: ShortenerError) -> Self {
        match error {
            ShortenerError::AliasConflict(code) => Self::AliasConflict(code),
            ShortenerError::InvalidUrl(message) => Self::InvalidUrl(message),
            ShortenerError::InvalidShortCode(message) => Self::InvalidShortCode(message),
            ShortenerError::Storage(message) => {
                if message.starts_with("storage backend unavailable:") {
                    Self::StorageUnavailable(message)
                } else if message.starts_with("storage operation timed out:") {
                    Self::StorageTimeout(message)
                } else {
                    Self::Internal(message)
                }
            }
        }
    }
}

impl From<RedirectorError> for BackendError {
    fn from(error: RedirectorError) -> Self {
        match error {
            RedirectorError::ShortCodeRequired => {
                Self::InvalidShortCode("short code is required".to_string())
            }
            RedirectorError::ShortCodeMalformed(message) => Self::InvalidShortCode(message),
            RedirectorError::ShortCodeNotFound => Self::NotFound,
            RedirectorError::Storage(source) => {
                let message = source.to_string();

                if message.starts_with("storage backend unavailable:") {
                    Self::StorageUnavailable(message)
                } else if message.starts_with("storage operation timed out:") {
                    Self::StorageTimeout(message)
                } else {
                    Self::Internal(message)
                }
            }
        }
    }
}
