use std::sync::Arc;
use thiserror::Error;
use tonic::{Code, Status};
use wormhole_proto_schema::v1::ConversionError;
use wormhole_storage::StorageError;

#[derive(Debug, Error)]
pub(crate) enum RedirectorError {
    #[error("short code is required")]
    ShortCodeRequired,
    #[error("short code is malformed: {0}")]
    ShortCodeMalformed(String),
    #[error("short code not found")]
    ShortCodeNotFound,
    #[error("storage operation failed: {0}")]
    Storage(
        #[from]
        #[source]
        StorageError,
    ),
}

impl From<ConversionError> for RedirectorError {
    fn from(error: ConversionError) -> Self {
        RedirectorError::ShortCodeMalformed(error.to_string())
    }
}

impl From<RedirectorError> for Status {
    fn from(error: RedirectorError) -> Self {
        match error {
            RedirectorError::ShortCodeRequired => {
                Status::new(Code::InvalidArgument, "short code is required")
            }
            RedirectorError::ShortCodeMalformed(source) => {
                Status::new(Code::InvalidArgument, source)
            }
            RedirectorError::ShortCodeNotFound => {
                Status::new(Code::NotFound, "short code not found")
            }
            RedirectorError::Storage(source) => {
                let (code, message) = match &source {
                    StorageError::Unavailable(_) => {
                        (Code::Unavailable, "storage backend unavailable")
                    }
                    StorageError::Timeout(_) => {
                        (Code::DeadlineExceeded, "storage operation timed out")
                    }
                    StorageError::Conflict(_) => (Code::AlreadyExists, "short code already exists"),
                    StorageError::InvalidData(_)
                    | StorageError::Query(_)
                    | StorageError::Cache(_)
                    | StorageError::Operation(_) => (Code::Internal, "storage operation failed"),
                };

                status_with_source(code, message, source)
            }
        }
    }
}

fn status_with_source<E>(code: Code, message: &'static str, source: E) -> Status
where
    E: std::error::Error + Send + Sync + 'static,
{
    let mut status = Status::new(code, message);
    status.set_source(Arc::new(source));
    status
}
