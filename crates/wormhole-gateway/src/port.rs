use async_trait::async_trait;

use crate::error::Result;
use crate::model::{CreateUrlRequest, CreateUrlResponse, GetUrlResponse};

#[async_trait]
/// Write-side gateway boundary for URL lifecycle operations.
///
/// This trait keeps the application layer decoupled from concrete adapters
/// (database, RPC client, etc.) while still exposing the business use-cases.
pub trait UrlWritePort: Send + Sync + 'static {
    /// Creates a new short URL mapping from the provided request payload.
    async fn create(&self, request: CreateUrlRequest) -> Result<CreateUrlResponse>;

    /// Deletes an existing short URL mapping by its short code.
    async fn delete(&self, short_code: &str) -> Result<()>;
}

#[async_trait]
/// Read-side gateway boundary for URL lookup operations.
pub trait UrlReadPort: Send + Sync + 'static {
    /// Resolves a short code into its redirect response payload.
    async fn get(&self, short_code: &str) -> Result<GetUrlResponse>;
}

/// Convenience port that represents the full URL API surface.
///
/// Callers can depend on a single trait object when they need both read and
/// write behavior, without coupling to any concrete implementation.
pub trait UrlApiPort: UrlWritePort + UrlReadPort {}

impl<T> UrlApiPort for T where T: UrlWritePort + UrlReadPort {}
