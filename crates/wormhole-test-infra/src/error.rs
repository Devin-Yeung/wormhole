use std::result::Result as StdResult;
use thiserror::Error;

/// Errors that can occur when working with test infrastructure containers.
#[derive(Debug, Error)]
pub enum TestInfraError {
    #[error("Container error: {0}")]
    Container(#[from] testcontainers::TestcontainersError),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
}

/// A type alias for `Result` with `TestInfraError`.
pub type Result<T> = StdResult<T, TestInfraError>;
