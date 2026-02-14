use async_trait::async_trait;
use tracing::{debug, trace, warn};
use wormhole_core::{ReadRepository, ShortCode, StorageError, UrlCache, UrlRecord};

/// Type alias for repository results.
pub type Result<T> = std::result::Result<T, StorageError>;

/// A read-only repository decorator that adds caching.
///
/// This implementation composes any [`ReadRepository`] with any [`UrlCache`]
/// implementation to provide transparent caching. Read operations check the
/// cache first, falling back to the inner repository. Successful reads from
/// the inner repository are cached.
#[derive(Debug, Clone)]
pub struct CachedRepository<R, C> {
    inner: R,
    cache: C,
}

impl<R: ReadRepository, C: UrlCache> CachedRepository<R, C> {
    /// Creates a new cached repository decorator.
    ///
    /// # Arguments
    ///
    /// * `inner` - The underlying read-only repository implementation
    /// * `cache` - The cache implementation (e.g., [`RedisUrlCache`])
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use wormhole_redirector::{CachedRepository, RedisUrlCache};
    /// use wormhole_storage::InMemoryRepository;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let redis_client = redis::Client::open("redis://127.0.0.1:6379")?;
    /// let redis_conn = redis_client.get_multiplexed_async_connection().await?;
    ///
    /// let inner_repo = InMemoryRepository::new();
    /// let cache = RedisUrlCache::new(redis_conn);
    /// let cached_repo = CachedRepository::new(inner_repo, cache);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(inner: R, cache: C) -> Self {
        Self { inner, cache }
    }

    /// Returns a reference to the inner repository.
    pub fn inner(&self) -> &R {
        &self.inner
    }

    /// Returns a reference to the cache.
    pub fn cache(&self) -> &C {
        &self.cache
    }

    /// Invalidate a cached entry.
    ///
    /// This is useful when the underlying data may have changed
    /// and you want to ensure the next read fetches fresh data.
    pub async fn invalidate(&self, code: &ShortCode) -> Result<()> {
        trace!(code = %code, "Invalidating cache entry");
        self.cache
            .del(code)
            .await
            .map_err(|e| StorageError::Other(e.into()))
    }
}

#[async_trait]
impl<R: ReadRepository, C: UrlCache> ReadRepository for CachedRepository<R, C> {
    async fn get(&self, code: &ShortCode) -> Result<Option<UrlRecord>> {
        trace!(code = %code, "Fetching URL record with cache");

        // Use get_or_compute for single-flight semantics:
        // concurrent requests for the same key will coalesce into a single fetch
        self.cache
            .get_or_compute(code, move |c| {
                let code = c.clone();
                async move {
                    trace!(code = %code, "Cache miss, fetching from inner repository");
                    self.inner
                        .get(&code)
                        .await
                        .map_err(|e| wormhole_core::CacheError::Other(e.into()))
                }
            })
            .await
            .map_err(|e| StorageError::Other(e.into()))
    }

    async fn exists(&self, code: &ShortCode) -> Result<bool> {
        trace!(code = %code, "Checking existence via get");

        // Use get_url for existence check - if it returns Some, it exists
        match self.cache.get_url(code).await {
            Ok(Some(_)) => {
                debug!(code = %code, "Cache hit indicates code exists");
                return Ok(true);
            }
            Ok(None) => {
                trace!(code = %code, "Cache miss for existence check");
            }
            Err(e) => {
                warn!(code = %code, error = %e, "Cache error on existence check, falling back to inner repository");
            }
        }

        // Fall back to inner repository
        self.inner.exists(code).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::MokaUrlCache;
    use wormhole_core::Repository;
    use wormhole_storage::InMemoryRepository;

    fn code(s: &str) -> ShortCode {
        ShortCode::new_unchecked(s)
    }

    fn test_record(url: &str) -> UrlRecord {
        UrlRecord {
            original_url: url.to_string(),
            expire_at: None,
        }
    }

    fn test_service() -> (
        CachedRepository<InMemoryRepository, MokaUrlCache>,
        MokaUrlCache,
    ) {
        let inner = InMemoryRepository::new();
        let cache = MokaUrlCache::new();
        let cached = CachedRepository::new(inner, cache.clone());
        (cached, cache)
    }

    #[tokio::test]
    async fn get_from_inner_when_cache_miss() {
        let (cached, _cache) = test_service();
        let c = code("abc123");
        let record = test_record("https://example.com");

        // Insert into inner repository
        cached.inner().insert(&c, record.clone()).await.unwrap();

        // Should get from inner and populate cache
        let result = cached.get(&c).await.unwrap();
        assert_eq!(result, Some(record));
    }

    #[tokio::test]
    async fn get_from_cache_when_cache_hit() {
        let (cached, cache) = test_service();
        let c = code("abc123");
        let record = test_record("https://example.com");

        // Pre-populate cache
        cache.set_url(&c, &record).await.unwrap();

        // Should get from cache without hitting inner
        let result = cached.get(&c).await.unwrap();
        assert_eq!(result, Some(record));
    }

    #[tokio::test]
    async fn exists_true_when_in_cache() {
        let (cached, cache) = test_service();
        let c = code("abc123");
        let record = test_record("https://example.com");

        // Pre-populate cache
        cache.set_url(&c, &record).await.unwrap();

        // Should return true from cache via get_url
        assert!(cached.exists(&c).await.unwrap());
    }

    #[tokio::test]
    async fn exists_checks_inner_when_not_in_cache() {
        let (cached, _cache) = test_service();
        let c = code("abc123");
        let record = test_record("https://example.com");

        // Insert only into inner repository
        cached.inner().insert(&c, record).await.unwrap();

        // Should check inner repository
        assert!(cached.exists(&c).await.unwrap());
    }

    #[tokio::test]
    async fn get_populates_cache() {
        let (cached, cache) = test_service();
        let c = code("abc123");
        let record = test_record("https://example.com");

        // Insert into inner repository
        cached.inner().insert(&c, record.clone()).await.unwrap();

        // First get should populate cache
        let _ = cached.get(&c).await.unwrap();

        // Now cache should have the record
        let cached_record = cache.get_url(&c).await.unwrap();
        assert_eq!(cached_record, Some(record));
    }

    #[tokio::test]
    async fn invalidate_removes_from_cache() {
        let (cached, cache) = test_service();
        let c = code("abc123");
        let record = test_record("https://example.com");

        // Pre-populate cache
        cache.set_url(&c, &record).await.unwrap();
        assert!(cache.get_url(&c).await.unwrap().is_some());

        // Invalidate
        cached.invalidate(&c).await.unwrap();

        // Should be gone from cache
        assert!(cache.get_url(&c).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn invalidate_is_idempotent() {
        let (cached, _cache) = test_service();
        let c = code("abc123");

        // Invalidate non-existent key should not error
        cached.invalidate(&c).await.unwrap();
    }
}
