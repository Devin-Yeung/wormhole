use std::time::Duration;

use wormhole_core::{CacheError, ShortCode, UrlCache, UrlRecord};
use wormhole_redirector::cache::RedisHAUrlCache;
use wormhole_test_infra::redis::{RedisHA, RedisHAConfig};

/// Test fixture that manages a Redis HA environment using test-infra.
pub struct RedisHATestFixture {
    #[allow(dead_code)]
    redis_ha: RedisHA,
    service_name: String,
    sentinel_urls: Vec<String>,
}

impl RedisHATestFixture {
    /// Starts a new Redis HA environment with master, replicas, and sentinels.
    pub async fn start() -> Self {
        let config = RedisHAConfig::default();
        let service_name = config.service_name.clone();

        let redis_ha = RedisHA::new(config)
            .await
            .expect("Failed to start Redis HA environment");

        // Get sentinel addresses
        let sentinel_urls = redis_ha.sentinel_addresses().await;

        // Wait for sentinels to discover the topology and replicas to sync
        tokio::time::sleep(Duration::from_secs(2)).await;

        eprintln!("Sentinel URLs: {:?}", sentinel_urls);

        Self {
            redis_ha,
            service_name,
            sentinel_urls,
        }
    }

    /// Creates a new RedisHAUrlCache instance.
    pub fn create_cache(&self) -> Result<RedisHAUrlCache, CacheError> {
        RedisHAUrlCache::new(self.sentinel_urls.clone(), &self.service_name)
    }

    /// Creates a new RedisHAUrlCache instance with custom prefix.
    pub fn create_cache_with_prefix(
        &self,
        prefix: impl Into<String>,
    ) -> Result<RedisHAUrlCache, CacheError> {
        RedisHAUrlCache::with_prefix(self.sentinel_urls.clone(), &self.service_name, prefix)
    }
}

/// Helper function to create a test URL record.
fn create_test_record(url: impl Into<String>) -> UrlRecord {
    UrlRecord {
        original_url: url.into(),
        expire_at: None,
    }
}

#[tokio::test]
async fn test_redis_ha_cache_basic_get_set() {
    let fixture = RedisHATestFixture::start().await;
    let cache = fixture.create_cache().unwrap();

    for i in 0..100 {
        let code = ShortCode::new(format!("testcode{i}")).unwrap();
        let record = create_test_record(format!("https://example.com/{i}"));

        // Initially, the cache should be empty
        let result = cache.get_url(&code).await.unwrap();
        assert!(result.is_none(), "Cache should be empty initially");

        // Set the URL in cache (writes to master)
        cache.set_url(&code, &record).await.unwrap();
        eprintln!("Set completed successfully");
    }

    awaitility::at_most(Duration::from_secs(10))
        .poll_interval(Duration::from_millis(100))
        .until_async(|| async {
            let code = ShortCode::new("testcode50").unwrap();
            let result = cache.get_url(&code).await.unwrap();
            result.is_some()
        })
        .await;
}
