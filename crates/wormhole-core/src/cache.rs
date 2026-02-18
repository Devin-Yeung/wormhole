use crate::shortcode::ShortCode;
use crate::UrlRecord;
use async_trait::async_trait;
use std::future::Future;

pub type Result<T> = std::result::Result<T, crate::error::CacheError>;

/// A cache for URL records.
///
/// This trait provides a domain-specific caching abstraction for [`UrlRecord`]s,
/// using [`ShortCode`] as the key. Implementations can use Redis, in-memory
/// caches, or other storage backends.
#[async_trait]
pub trait UrlCache: Send + Sync + 'static {
    /// Get URL record from cache.
    ///
    /// Returns `Ok(None)` if the key is not in the cache.
    async fn get_url(&self, code: &ShortCode) -> Result<Option<UrlRecord>>;

    /// Store URL record in cache.
    async fn set_url(&self, code: &ShortCode, record: &UrlRecord) -> Result<()>;

    /// Remove URL record from cache.
    ///
    /// Essential for handling URL updates or deletions.
    /// It is not an error if the key does not exist.
    async fn del(&self, code: &ShortCode) -> Result<()>;

    /// Get URL record from cache, computing it if not present.
    ///
    /// This method provides a way to atomically fetch or compute a cached value.
    /// Implementations that support single-flight (request coalescing) should
    /// override this to ensure concurrent requests for the same key only result
    /// in a single computation.
    ///
    /// The default implementation simply delegates to [`get_url`](Self::get_url)
    /// and calls `fetch` on cache miss, without any coalescing guarantees.
    async fn get_or_compute<F, Fut>(&self, code: &ShortCode, fetch: F) -> Result<Option<UrlRecord>>
    where
        F: FnOnce(&ShortCode) -> Fut + Send,
        Fut: Future<Output = Result<Option<UrlRecord>>> + Send,
    {
        match self.get_url(code).await? {
            Some(record) => Ok(Some(record)),
            None => {
                let record = fetch(code).await?;
                if let Some(ref r) = record {
                    self.set_url(code, r).await?;
                }
                Ok(record)
            }
        }
    }
}
