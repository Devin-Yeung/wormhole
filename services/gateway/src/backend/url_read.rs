use super::Result;
use async_trait::async_trait;
use jiff::Timestamp;

#[derive(Debug, Clone)]
pub struct GetUrlCmd {
    pub short_code: String,
}

#[derive(Debug, Clone)]
pub struct GetUrlResult {
    pub original_url: String,
    pub expire_at: Option<Timestamp>,
}

#[async_trait]
/// Read-side gateway boundary for URL lookup operations.
pub trait UrlRead: Send + Sync + 'static {
    /// Resolves a short code into its redirect response payload.
    async fn get(&self, short_code: &str) -> Result<GetUrlResult>;
}
