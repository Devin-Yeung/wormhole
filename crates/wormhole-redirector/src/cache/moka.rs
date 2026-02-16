use async_trait::async_trait;
use moka::future::Cache;
use std::future::Future;
use std::time::Duration;
use tracing::{debug, trace};
use typed_builder::TypedBuilder;
use wormhole_core::{CacheError, ShortCode, UrlCache, UrlRecord};

/// Type alias for cache results.
pub type Result<T> = std::result::Result<T, CacheError>;

/// An in-memory cache implementation using Moka.
///
/// This implementation stores URL records in a concurrent, high-performance
/// in-memory cache. It's ideal for single-node deployments or as a L1 cache
/// in front of Redis.
#[derive(Debug, Clone)]
pub struct MokaUrlCache {
    // Use Option<UrlRecord> to properly handle "not found" cases in single-flight
    cache: Cache<String, Option<UrlRecord>>,
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
    pub fn builder() -> CacheConfigBuilder {
        CacheConfig::builder()
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
                Ok(record)
            }
            None => {
                trace!(code = %code, "Cache miss in Moka");
                Ok(None)
            }
        }
    }

    async fn set_url(&self, code: &ShortCode, record: &UrlRecord) -> Result<()> {
        trace!(code = %code, "Storing URL record in Moka cache");

        let key = code.as_str().to_string();
        self.cache.insert(key, Some(record.clone())).await;
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

    async fn get_or_compute<F, Fut>(&self, code: &ShortCode, fetch: F) -> Result<Option<UrlRecord>>
    where
        F: FnOnce(&ShortCode) -> Fut + Send,
        Fut: Future<Output = Result<Option<UrlRecord>>> + Send,
    {
        trace!(code = %code, "Fetching URL record from Moka cache with single-flight");

        let key = code.as_str().to_string();

        // Moka's try_get_with provides single-flight semantics:
        // concurrent requests for the same key will coalesce into a single fetch
        let result = self
            .cache
            .try_get_with(key, async {
                trace!(code = %code, "Cache miss, performing single-flight fetch");
                fetch(code).await
            })
            .await
            .map_err(|e| e.as_ref().clone())?;

        debug!(code = %code, "Single-flight fetch completed");
        Ok(result)
    }
}

/// Configuration for creating a MokaUrlCache with custom settings.
#[derive(Debug, TypedBuilder, Default)]
pub struct CacheConfig {
    /// Maximum number of entries the cache can hold.
    #[builder(default, setter(strip_option))]
    max_capacity: Option<u64>,
    /// Time-to-live for cache entries.
    #[builder(default, setter(strip_option))]
    ttl: Option<Duration>,
    /// Time-to-idle for cache entries.
    #[builder(default, setter(strip_option))]
    tti: Option<Duration>,
}

impl From<CacheConfig> for MokaUrlCache {
    fn from(config: CacheConfig) -> Self {
        let mut builder = Cache::builder();

        if let Some(capacity) = config.max_capacity {
            builder = builder.max_capacity(capacity);
        }

        if let Some(ttl) = config.ttl {
            builder = builder.time_to_live(ttl);
        }

        if let Some(tti) = config.tti {
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
        cache.set_url(&c, &record).await.unwrap();

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
        cache.set_url(&c, &record).await.unwrap();
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
        cache.set_url(&c, &record).await.unwrap();
        assert!(cache.get_url(&c).await.unwrap().is_some());

        // Wait for TTL to expire
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Should be gone
        assert!(cache.get_url(&c).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn cache_builder_pattern() {
        let cache: MokaUrlCache = MokaUrlCache::builder()
            .max_capacity(1000)
            .ttl(Duration::from_secs(60))
            .tti(Duration::from_secs(30))
            .build()
            .into();

        let c = code("abc123");
        let record = test_record("https://example.com");

        cache.set_url(&c, &record).await.unwrap();
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
            cache.set_url(&c, &record).await.unwrap();
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

        cache.set_url(&c, &record).await.unwrap();

        // Get the record twice - should get independent clones
        let r1 = cache.get_url(&c).await.unwrap().unwrap();
        let r2 = cache.get_url(&c).await.unwrap().unwrap();

        assert_eq!(r1.original_url, r2.original_url);
        assert_eq!(r1.expire_at, r2.expire_at);
    }

    #[tokio::test]
    async fn single_flight_prevents_concurrent_fetch() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let cache = MokaUrlCache::new();
        let fetch_count = Arc::new(AtomicUsize::new(0));

        // Spawn 10 concurrent requests for the same key
        let mut handles = vec![];
        for _ in 0..10 {
            let cache = cache.clone();
            let c = code("abc123");
            let count = fetch_count.clone();
            handles.push(tokio::spawn(async move {
                cache
                    .get_or_compute(&c, |_code| async {
                        // Simulate slow fetch
                        tokio::time::sleep(Duration::from_millis(50)).await;
                        count.fetch_add(1, Ordering::SeqCst);
                        Ok(Some(test_record("https://example.com")))
                    })
                    .await
            }));
        }

        // Wait for all to complete
        for handle in handles {
            let _ = handle.await.unwrap();
        }

        // The fetch should only have been called once due to single-flight
        assert_eq!(
            fetch_count.load(Ordering::SeqCst),
            1,
            "Single-flight should prevent concurrent fetches for the same key"
        );
    }

    #[tokio::test]
    async fn single_flight_different_keys_fetch_independently() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let cache = MokaUrlCache::new();
        let fetch_count = Arc::new(AtomicUsize::new(0));

        // Spawn requests for different keys
        let mut handles = vec![];
        for i in 0..5 {
            let cache = cache.clone();
            let c = code(&format!("code{}", i));
            let count = fetch_count.clone();
            handles.push(tokio::spawn(async move {
                cache
                    .get_or_compute(&c, |_code| async {
                        tokio::time::sleep(Duration::from_millis(10)).await;
                        count.fetch_add(1, Ordering::SeqCst);
                        Ok(Some(test_record(&format!("https://example{}", i))))
                    })
                    .await
            }));
        }

        // Wait for all to complete
        for handle in handles {
            let _ = handle.await.unwrap();
        }

        // Each key should have been fetched independently
        assert_eq!(
            fetch_count.load(Ordering::SeqCst),
            5,
            "Different keys should be fetched independently"
        );
    }

    #[tokio::test]
    async fn single_flight_propagates_fetch_error() {
        let cache = MokaUrlCache::new();
        let c = code("abc123");

        let err = cache
            .get_or_compute(&c, |_code| async {
                Err(CacheError::Timeout("simulated timeout".to_string()))
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CacheError::Timeout(_)));
    }
}
