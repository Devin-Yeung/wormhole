use async_trait::async_trait;
use moka::future::Cache;
use std::time::Duration;
use tracing::{debug, trace};
use wormhole_core::{Result, ShortCode, UrlCache, UrlRecord};

/// An in-memory cache implementation using Moka.
///
/// This implementation stores URL records in a concurrent, high-performance
/// in-memory cache. It's ideal for single-node deployments or as a L1 cache
/// in front of Redis.
#[derive(Debug, Clone)]
pub struct MokaUrlCache {
    cache: Cache<String, UrlRecord>,
}

impl MokaUrlCache {
    /// Creates a new Moka URL cache with default settings.
    ///
    /// The cache will have a default maximum capacity of 10,000 entries.
    pub fn new() -> Self {
        let cache = Cache::builder().max_capacity(10_000).build();
        Self { cache }
    }

    /// Creates a new Moka URL cache with a custom maximum capacity.
    ///
    /// # Arguments
    ///
    /// * `max_capacity` - Maximum number of entries the cache can hold
    pub fn with_capacity(max_capacity: u64) -> Self {
        let cache = Cache::builder().max_capacity(max_capacity).build();
        Self { cache }
    }

    /// Creates a new Moka URL cache with time-to-live (TTL) settings.
    ///
    /// Entries will expire after the specified TTL from the time of insertion.
    ///
    /// # Arguments
    ///
    /// * `max_capacity` - Maximum number of entries the cache can hold
    /// * `ttl` - Time-to-live for cache entries
    pub fn with_ttl(max_capacity: u64, ttl: Duration) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(ttl)
            .build();
        Self { cache }
    }

    /// Creates a new Moka URL cache with time-to-idle (TTI) settings.
    ///
    /// Entries will expire if they are not accessed for the specified duration.
    ///
    /// # Arguments
    ///
    /// * `max_capacity` - Maximum number of entries the cache can hold
    /// * `tti` - Time-to-idle for cache entries
    pub fn with_tti(max_capacity: u64, tti: Duration) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_capacity)
            .time_to_idle(tti)
            .build();
        Self { cache }
    }

    /// Returns a builder for creating a custom cache configuration.
    pub fn builder() -> CacheBuilder {
        CacheBuilder::new()
    }
}

impl Default for MokaUrlCache {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UrlCache for MokaUrlCache {
    async fn get_url(&self, code: &ShortCode) -> Result<Option<UrlRecord>> {
        trace!(code = %code, "Fetching URL record from Moka cache");

        let key = code.as_str().to_string();
        match self.cache.get(&key).await {
            Some(record) => {
                debug!(code = %code, "Cache hit in Moka");
                Ok(Some(record))
            }
            None => {
                trace!(code = %code, "Cache miss in Moka");
                Ok(None)
            }
        }
    }

    async fn set_url(
        &self,
        code: &ShortCode,
        record: &UrlRecord,
        _ttl: Option<Duration>,
    ) -> Result<()> {
        trace!(code = %code, "Storing URL record in Moka cache");

        let key = code.as_str().to_string();
        self.cache.insert(key, record.clone()).await;
        debug!(code = %code, "Cached record in Moka");
        Ok(())
    }

    async fn del(&self, code: &ShortCode) -> Result<()> {
        trace!(code = %code, "Removing URL record from Moka cache");

        let key = code.as_str().to_string();
        self.cache.invalidate(&key).await;
        debug!(code = %code, "Removed record from Moka cache (if present)");
        Ok(())
    }
}

/// A builder for creating a MokaUrlCache with custom configuration.
#[derive(Debug, Default)]
pub struct CacheBuilder {
    max_capacity: Option<u64>,
    ttl: Option<Duration>,
    tti: Option<Duration>,
}

impl CacheBuilder {
    /// Creates a new cache builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum capacity of the cache.
    pub fn max_capacity(mut self, capacity: u64) -> Self {
        self.max_capacity = Some(capacity);
        self
    }

    /// Sets the time-to-live for cache entries.
    pub fn ttl(mut self, ttl: Duration) -> Self {
        self.ttl = Some(ttl);
        self
    }

    /// Sets the time-to-idle for cache entries.
    pub fn tti(mut self, tti: Duration) -> Self {
        self.tti = Some(tti);
        self
    }

    /// Builds the MokaUrlCache with the configured settings.
    pub fn build(self) -> MokaUrlCache {
        let mut builder = Cache::builder();

        if let Some(capacity) = self.max_capacity {
            builder = builder.max_capacity(capacity);
        }

        if let Some(ttl) = self.ttl {
            builder = builder.time_to_live(ttl);
        }

        if let Some(tti) = self.tti {
            builder = builder.time_to_idle(tti);
        }

        MokaUrlCache {
            cache: builder.build(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::Timestamp;

    fn test_record(url: &str) -> UrlRecord {
        UrlRecord {
            original_url: url.to_string(),
            expire_at: None,
        }
    }

    fn code(s: &str) -> ShortCode {
        ShortCode::new_unchecked(s)
    }

    #[tokio::test]
    async fn cache_get_and_set() {
        let cache = MokaUrlCache::new();
        let c = code("abc123");
        let record = test_record("https://example.com");

        // Initially empty
        assert!(cache.get_url(&c).await.unwrap().is_none());

        // Insert
        cache.set_url(&c, &record, None).await.unwrap();

        // Now it should exist
        let result = cache.get_url(&c).await.unwrap();
        assert_eq!(result, Some(record));
    }

    #[tokio::test]
    async fn cache_del_removes_entry() {
        let cache = MokaUrlCache::new();
        let c = code("abc123");
        let record = test_record("https://example.com");

        // Insert
        cache.set_url(&c, &record, None).await.unwrap();
        assert!(cache.get_url(&c).await.unwrap().is_some());

        // Delete
        cache.del(&c).await.unwrap();

        // Should be gone
        assert!(cache.get_url(&c).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn cache_del_is_idempotent() {
        let cache = MokaUrlCache::new();
        let c = code("abc123");

        // Delete non-existent key should not error
        cache.del(&c).await.unwrap();

        // Still not there
        assert!(cache.get_url(&c).await.unwrap().is_none());

        // Delete again is still fine
        cache.del(&c).await.unwrap();
    }

    #[tokio::test]
    async fn cache_with_ttl_expires() {
        let cache = MokaUrlCache::with_ttl(100, Duration::from_millis(50));
        let c = code("abc123");
        let record = test_record("https://example.com");

        // Insert
        cache.set_url(&c, &record, None).await.unwrap();
        assert!(cache.get_url(&c).await.unwrap().is_some());

        // Wait for TTL to expire
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Should be gone
        assert!(cache.get_url(&c).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn cache_builder_pattern() {
        let cache = MokaUrlCache::builder()
            .max_capacity(1000)
            .ttl(Duration::from_secs(60))
            .tti(Duration::from_secs(30))
            .build();

        let c = code("abc123");
        let record = test_record("https://example.com");

        cache.set_url(&c, &record, None).await.unwrap();
        assert!(cache.get_url(&c).await.unwrap().is_some());
    }

    #[tokio::test]
    async fn cache_handles_many_entries() {
        // Test that the cache can handle many entries without issues
        let cache = MokaUrlCache::with_capacity(100);

        // Insert entries
        for i in 0..50 {
            let c = code(&format!("code{}", i));
            let record = test_record(&format!("https://example{}", i));
            cache.set_url(&c, &record, None).await.unwrap();
        }

        // Verify some entries exist
        let c0 = code("code0");
        let c25 = code("code25");
        let c49 = code("code49");

        assert!(cache.get_url(&c0).await.unwrap().is_some());
        assert!(cache.get_url(&c25).await.unwrap().is_some());
        assert!(cache.get_url(&c49).await.unwrap().is_some());

        // Verify their records are correct
        assert_eq!(
            cache.get_url(&c0).await.unwrap().unwrap().original_url,
            "https://example0"
        );
        assert_eq!(
            cache.get_url(&c25).await.unwrap().unwrap().original_url,
            "https://example25"
        );
    }

    #[tokio::test]
    async fn cache_clones_record() {
        let cache = MokaUrlCache::new();
        let c = code("abc123");
        let record = UrlRecord {
            original_url: "https://example.com".to_string(),
            expire_at: Some(Timestamp::now()),
        };

        cache.set_url(&c, &record, None).await.unwrap();

        // Get the record twice - should get independent clones
        let r1 = cache.get_url(&c).await.unwrap().unwrap();
        let r2 = cache.get_url(&c).await.unwrap().unwrap();

        assert_eq!(r1.original_url, r2.original_url);
        assert_eq!(r1.expire_at, r2.expire_at);
    }
}
