use super::Result;
use async_trait::async_trait;
use jiff::Timestamp;

#[derive(Debug, Clone)]
pub struct WriteUrlCmd {
    pub original_url: String,
    pub custom_alias: Option<String>,
    pub expire_at: Option<Timestamp>,
}

#[derive(Debug, Clone)]
pub struct WriteUrlResult {
    pub short_code: String,
    pub short_url: String,
    pub original_url: String,
    pub expire_at: Option<Timestamp>,
}

#[derive(Debug, Clone)]
pub struct DeleteUrlCmd {
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
    async fn delete(&self, short_code: &str) -> Result<()>;
}
