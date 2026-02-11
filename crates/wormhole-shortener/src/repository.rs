use crate::error::Result;
use async_trait::async_trait;
use jiff::Timestamp;

#[async_trait]
pub trait Repository: Send + Sync + 'static {
    /// Saves a new URL mapping to the repository.
    async fn save(&self, id: u64, original_url: &str, expire_at: Option<Timestamp>) -> Result<()>;
    /// Retrieves the original URL for a given ID.
    async fn get(&self, id: u64) -> Result<Option<String>>;
}
