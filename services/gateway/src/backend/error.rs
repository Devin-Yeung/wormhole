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
