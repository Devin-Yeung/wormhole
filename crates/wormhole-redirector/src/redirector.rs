use crate::Result;
use async_trait::async_trait;
use wormhole_core::{ShortCode, UrlRecord};

#[async_trait]
pub trait Redirector: Send + Sync + 'static {
    /// Resolves a short code to its stored URL record.
    /// Returns `None` if the code does not exist or has expired.
    async fn resolve(&self, code: &ShortCode) -> Result<Option<UrlRecord>>;
}
