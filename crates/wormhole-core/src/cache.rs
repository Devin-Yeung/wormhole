use crate::error::Result;
use crate::repository::UrlRecord;
use crate::shortcode::ShortCode;
use async_trait::async_trait;
use std::time::Duration;

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

    /// Store URL record in cache with optional TTL.
    ///
    /// If `ttl` is `None`, the entry may persist indefinitely or use
    /// a default expiration policy depending on the implementation.
    async fn set_url(
        &self,
        code: &ShortCode,
        record: &UrlRecord,
        ttl: Option<Duration>,
    ) -> Result<()>;

    /// Check if short code exists in cache.
    ///
    /// Returns `true` if the code exists in the cache, `false` otherwise.
    /// Note that this does not check the underlying repository.
    async fn exists(&self, code: &ShortCode) -> Result<bool>;
}
