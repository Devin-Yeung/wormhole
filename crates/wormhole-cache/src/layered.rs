use async_trait::async_trait;
use std::future::Future;
use tracing::{debug, trace};
use wormhole_core::{CacheError, ShortCode, UrlRecord};

use crate::UrlCache;

/// Type alias for cache results.
pub type Result<T> = std::result::Result<T, CacheError>;

/// A multi-layer cache that composes two cache implementations.
///
/// This cache implementation provides a two-level caching strategy where
/// L1 is typically a fast, local cache (e.g., Moka in-memory cache) and
/// L2 is typically a slower, distributed cache (e.g., Redis).
///
/// # Operation Strategy
///
/// - **Get**: Try L1 first, if miss try L2. If L2 has the value, populate L1
///   with it (cache-aside pattern with backfill).
/// - **Set**: Write to both L1 and L2 (write-through pattern).
/// - **Delete**: Remove from both L1 and L2.
///
/// # Type Parameters
///
/// * `L1` - The primary/faster cache (e.g., `MokaUrlCache`)
/// * `L2` - The secondary/slower cache (e.g., `RedisUrlCache`)
///
/// # Example
///
/// ```rust
/// use wormhole_cache::{LayeredCache, MokaUrlCache};
///
/// // Create L1 cache (in-memory)
/// let l1 = MokaUrlCache::with_capacity(10_000);
///
/// // Create L2 cache (e.g., Redis)
/// // let l2 = RedisUrlCache::new(redis_client);
///
/// // Compose them into a layered cache
/// // let cache = LayeredCache::new(l1, l2);
/// ```
#[derive(Debug, Clone)]
pub struct LayeredCache<L1, L2> {
    l1: L1,
    l2: L2,
}

impl<L1, L2> LayeredCache<L1, L2> {
    /// Creates a new layered cache with the given L1 and L2 caches.
    ///
    /// # Arguments
    ///
    /// * `l1` - The primary/faster cache
    /// * `l2` - The secondary/slower cache
    pub fn new(l1: L1, l2: L2) -> Self {
        Self { l1, l2 }
    }

    /// Returns a reference to the L1 cache.
    pub fn l1(&self) -> &L1 {
        &self.l1
    }

    /// Returns a reference to the L2 cache.
    pub fn l2(&self) -> &L2 {
        &self.l2
    }

    /// Consumes the layered cache and returns the inner caches.
    pub fn into_inner(self) -> (L1, L2) {
        (self.l1, self.l2)
    }
}

impl<L1, L2> LayeredCache<L1, L2>
where
    L1: UrlCache,
    L2: UrlCache,
{
    /// Get URL record from cache, computing it if not present.
    ///
    /// This method delegates to L1's [`get_or_compute`](UrlCache::get_or_compute)
    /// to take advantage of single-flight semantics. If L1 doesn't have the value,
    /// it will try L2 before calling `fetch`.
    ///
    /// This is useful for preventing cache stampedes (thundering herd) when
    /// concurrent requests all miss the cache simultaneously.
    pub async fn get_or_compute<F, Fut>(
        &self,
        code: &ShortCode,
        fetch: F,
    ) -> Result<Option<UrlRecord>>
    where
        F: FnOnce(&ShortCode) -> Fut + Send,
        Fut: Future<Output = Result<Option<UrlRecord>>> + Send,
    {
        trace!(code = %code, "Fetching URL record from layered cache with single-flight");

        // Clone for move closure
        let code = code.clone();
        let l2 = &self.l2;

        // Chain single-flight: L1.get_or_compute wraps L2.get_or_compute wraps fetch
        // This ensures both layers' single-flight semantics are respected
        self.l1
            .get_or_compute(&code, move |c| {
                let c = c.clone();
                async move { l2.get_or_compute(&c, fetch).await }
            })
            .await
    }
}

#[async_trait]
impl<L1, L2> UrlCache for LayeredCache<L1, L2>
where
    L1: UrlCache,
    L2: UrlCache,
{
    async fn get_url(&self, code: &ShortCode) -> Result<Option<UrlRecord>> {
        trace!(code = %code, "Fetching URL record from layered cache");

        // Try L1 first
        match self.l1.get_url(code).await? {
            Some(record) => {
                debug!(code = %code, "L1 cache hit");
                return Ok(Some(record));
            }
            None => {
                trace!(code = %code, "L1 cache miss, trying L2");
            }
        }

        // L1 miss, try L2
        match self.l2.get_url(code).await? {
            Some(record) => {
                debug!(code = %code, "L2 cache hit, backfilling L1");
                // Backfill L1 with the record from L2 so subsequent reads stay local.
                self.l1.set_url(code, &record).await?;
                Ok(Some(record))
            }
            None => {
                trace!(code = %code, "L2 cache miss");
                Ok(None)
            }
        }
    }

    async fn set_url(&self, code: &ShortCode, record: &UrlRecord) -> Result<()> {
        trace!(code = %code, "Storing URL record in layered cache");

        // Write to L2 first (slower, more durable), then L1
        self.l2.set_url(code, record).await?;
        debug!(code = %code, "Stored in L2 cache");

        // Also write to L1
        self.l1.set_url(code, record).await?;
        debug!(code = %code, "Stored in L1 cache");

        Ok(())
    }

    async fn del(&self, code: &ShortCode) -> Result<()> {
        trace!(code = %code, "Removing URL record from layered cache");

        // Delete from both caches
        // We delete from L1 first (fast), then L2
        self.l1.del(code).await?;
        debug!(code = %code, "Removed from L1 cache");

        self.l2.del(code).await?;
        debug!(code = %code, "Removed from L2 cache");

        Ok(())
    }

    async fn get_or_compute<F, Fut>(&self, code: &ShortCode, fetch: F) -> Result<Option<UrlRecord>>
    where
        F: FnOnce(&ShortCode) -> Fut + Send,
        Fut: Future<Output = Result<Option<UrlRecord>>> + Send,
    {
        // Delegate to the inherent method to share the implementation
        self.get_or_compute(code, fetch).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MokaUrlCache;
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

    fn create_test_cache() -> LayeredCache<MokaUrlCache, MokaUrlCache> {
        let l1 = MokaUrlCache::with_capacity(100);
        let l2 = MokaUrlCache::with_capacity(100);
        LayeredCache::new(l1, l2)
    }

    #[tokio::test]
    async fn layered_cache_get_from_l1() {
        let cache = create_test_cache();
        let c = code("abc123");
        let record = test_record("https://example.com");

        // Insert directly into L1
        cache.l1.set_url(&c, &record).await.unwrap();

        // Should get from L1
        let result = cache.get_url(&c).await.unwrap();
        assert_eq!(result, Some(record));
    }

    #[tokio::test]
    async fn layered_cache_get_backfills_l1_from_l2() {
        let cache = create_test_cache();
        let c = code("abc123");
        let record = test_record("https://example.com");

        // Insert only into L2
        cache.l2.set_url(&c, &record).await.unwrap();

        // L1 should be empty initially
        assert!(cache.l1.get_url(&c).await.unwrap().is_none());

        // Get from layered cache should find it in L2 and backfill L1
        let result = cache.get_url(&c).await.unwrap();
        assert_eq!(result, Some(record.clone()));

        // Now L1 should have it
        assert_eq!(cache.l1.get_url(&c).await.unwrap(), Some(record));
    }

    #[tokio::test]
    async fn layered_cache_set_writes_to_both() {
        let cache = create_test_cache();
        let c = code("abc123");
        let record = test_record("https://example.com");

        // Set through layered cache
        cache.set_url(&c, &record).await.unwrap();

        // Should be in both caches
        assert_eq!(cache.l1.get_url(&c).await.unwrap(), Some(record.clone()));
        assert_eq!(cache.l2.get_url(&c).await.unwrap(), Some(record));
    }

    #[tokio::test]
    async fn layered_cache_del_removes_from_both() {
        let cache = create_test_cache();
        let c = code("abc123");
        let record = test_record("https://example.com");

        // Insert into both caches
        cache.l1.set_url(&c, &record).await.unwrap();
        cache.l2.set_url(&c, &record).await.unwrap();

        // Delete through layered cache
        cache.del(&c).await.unwrap();

        // Should be removed from both caches
        assert!(cache.l1.get_url(&c).await.unwrap().is_none());
        assert!(cache.l2.get_url(&c).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn layered_cache_miss_when_both_empty() {
        let cache = create_test_cache();
        let c = code("abc123");

        // Should return None when both caches miss
        let result = cache.get_url(&c).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn layered_cache_backfill_with_expiration() {
        let cache = create_test_cache();
        let c = code("abc123");

        // Create record with future expiration
        let future_time = Timestamp::now() + jiff::SignedDuration::from_secs(3600);
        let record = UrlRecord {
            original_url: "https://example.com".to_string(),
            expire_at: Some(future_time),
        };

        // Insert only into L2
        cache.l2.set_url(&c, &record).await.unwrap();

        // Get should backfill L1
        let result = cache.get_url(&c).await.unwrap();
        assert_eq!(result, Some(record.clone()));

        // L1 should now have the record
        assert_eq!(cache.l1.get_url(&c).await.unwrap(), Some(record));
    }

    #[tokio::test]
    async fn layered_cache_del_is_idempotent() {
        let cache = create_test_cache();
        let c = code("abc123");

        // Delete non-existent key should not error
        cache.del(&c).await.unwrap();

        // Still not there
        assert!(cache.get_url(&c).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn layered_cache_into_inner() {
        let l1 = MokaUrlCache::with_capacity(100);
        let l2 = MokaUrlCache::with_capacity(200);

        let cache = LayeredCache::new(l1, l2);
        let (inner_l1, _inner_l2) = cache.into_inner();

        // We can't directly compare the caches, but we can verify they work
        let c = code("abc123");
        let record = test_record("https://example.com");

        inner_l1.set_url(&c, &record).await.unwrap();
        assert_eq!(inner_l1.get_url(&c).await.unwrap(), Some(record));
    }

    #[tokio::test]
    async fn layered_cache_single_flight_uses_l1_semantics() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;
        use std::time::Duration;

        let cache = create_test_cache();
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

        // The fetch should only have been called once due to L1's single-flight
        assert_eq!(
            fetch_count.load(Ordering::SeqCst),
            1,
            "Layered cache should use L1's single-flight semantics"
        );
    }

    #[tokio::test]
    async fn layered_cache_single_flight_skips_l2_when_cached() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;
        use std::time::Duration;

        let cache = create_test_cache();
        let fetch_count = Arc::new(AtomicUsize::new(0));
        let c = code("abc123");
        let record = test_record("https://example.com");

        // Pre-populate L2
        cache.l2.set_url(&c, &record).await.unwrap();

        // Spawn concurrent requests
        let mut handles = vec![];
        for _ in 0..5 {
            let cache = cache.clone();
            let c = code("abc123");
            let count = fetch_count.clone();
            handles.push(tokio::spawn(async move {
                cache
                    .get_or_compute(&c, |_code| async {
                        // This fetch should not be called since L2 has the value
                        tokio::time::sleep(Duration::from_millis(10)).await;
                        count.fetch_add(1, Ordering::SeqCst);
                        Ok(Some(test_record("https://fetched.com")))
                    })
                    .await
            }));
        }

        // Wait for all to complete
        for handle in handles {
            let result = handle.await.unwrap().unwrap();
            // Should get the value from L2, not from fetch
            assert_eq!(result.unwrap().original_url, "https://example.com");
        }

        // Fetch should not have been called since L2 had the value
        assert_eq!(
            fetch_count.load(Ordering::SeqCst),
            0,
            "Fetch should not be called when L2 has the value"
        );

        // L1 should now be backfilled
        assert_eq!(
            cache.l1.get_url(&c).await.unwrap().unwrap().original_url,
            "https://example.com"
        );
    }
}
