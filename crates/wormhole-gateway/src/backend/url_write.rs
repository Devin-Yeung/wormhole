use super::Result;
use async_trait::async_trait;
use jiff::Timestamp;

/// Command for creating a new short URL.
#[derive(Debug, Clone)]
pub struct WriteUrlCmd {
    /// The original URL to be shortened.
    pub original_url: String,
    /// Optional user-provided alias (must be unique).
    pub custom_alias: Option<String>,
    /// Optional expiration timestamp. If `None`, the URL never expires.
    pub expire_at: Option<Timestamp>,
}

/// Result of a successful URL creation.
#[derive(Debug, Clone)]
pub struct WriteUrlResult {
    /// The generated or custom short code.
    pub short_code: String,
    /// The full short URL for sharing.
    pub short_url: String,
    /// Echo back the original URL for confirmation.
    pub original_url: String,
    /// The expiration timestamp (may differ from input if adjusted).
    pub expire_at: Option<Timestamp>,
}

/// Command for deleting a short URL.
#[derive(Debug, Clone)]
pub struct DeleteUrlCmd {
    /// The short code to delete.
    pub short_code: String,
}

#[async_trait]
/// Write-side gateway boundary for URL lifecycle operations.
///
/// This trait keeps the application layer decoupled from concrete adapters
/// (database, RPC client, etc.) while still exposing the business use-cases.
pub trait UrlWrite: Send + Sync + 'static {
    /// Creates a new short URL mapping from the provided request payload.
    async fn create(&self, cmd: WriteUrlCmd) -> Result<WriteUrlResult>;

    /// Deletes an existing short URL mapping by its short code.
    async fn delete(&self, cmd: DeleteUrlCmd) -> Result<()>;
}
