use std::time::Duration;

use redis::AsyncCommands;
use wormhole_core::{ShortCode, UrlCache, UrlRecord};
use wormhole_redirector::cache::RedisUrlCache;
use wormhole_test_infra::redis::RedisMaster;

/// Test fixture that manages a Redis container using test-infra.
pub struct RedisTestContainer {
    #[allow(dead_code)]
    redis: RedisMaster,
    redis_url: String,
}

impl RedisTestContainer {
    /// Starts a new Redis container with a random available port.
    pub async fn start() -> Self {
        let redis = RedisMaster::new()
            .await
            .expect("Failed to start Redis master");
        let host = redis.host().await.expect("Failed to get Redis host");
        let port = redis.port().await.expect("Failed to get Redis port");
        let redis_url = format!("redis://{}:{}", host, port);

        // Wait a moment to ensure Redis is fully ready
        tokio::time::sleep(Duration::from_millis(500)).await;

        Self { redis, redis_url }
    }

    /// Creates a new Redis connection.
    pub async fn create_connection(&self) -> redis::aio::MultiplexedConnection {
        let client =
            redis::Client::open(self.redis_url.as_str()).expect("Failed to create Redis client");
        client
            .get_multiplexed_async_connection()
            .await
            .expect("Failed to get Redis connection")
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
async fn test_redis_cache_basic_get_set() {
    let fixture = RedisTestContainer::start().await;
    let conn = fixture.create_connection().await;
    let cache = RedisUrlCache::new(conn);

    let code = ShortCode::new("test123").unwrap();
    let record = create_test_record("https://example.com");

    // Initially, the cache should be empty
    let result = cache.get_url(&code).await.unwrap();
    assert!(result.is_none(), "Cache should be empty initially");

    // Set the URL in cache
    cache.set_url(&code, &record).await.unwrap();

    // Now we should be able to get it back
    let result = cache.get_url(&code).await.unwrap();
    assert!(
        result.is_some(),
        "Cache should contain the record after set"
    );
    let cached_record = result.unwrap();
    assert_eq!(cached_record.original_url, "https://example.com");
}

#[tokio::test]
async fn test_redis_cache_delete() {
    let fixture = RedisTestContainer::start().await;
    let conn = fixture.create_connection().await;
    let cache = RedisUrlCache::new(conn);

    let code = ShortCode::new("delete123").unwrap();
    let record = create_test_record("https://example.com/delete");

    // Set the URL in cache
    cache.set_url(&code, &record).await.unwrap();

    // Verify it's there
    let result = cache.get_url(&code).await.unwrap();
    assert!(result.is_some());

    // Delete it
    cache.del(&code).await.unwrap();

    // Verify it's gone
    let result = cache.get_url(&code).await.unwrap();
    assert!(result.is_none(), "Cache should be empty after delete");
}

#[tokio::test]
async fn test_redis_cache_multiple_codes() {
    let fixture = RedisTestContainer::start().await;
    let conn = fixture.create_connection().await;
    let cache = RedisUrlCache::new(conn);

    let code1 = ShortCode::new("abc123").unwrap();
    let code2 = ShortCode::new("def456").unwrap();
    let record1 = create_test_record("https://example.com/1");
    let record2 = create_test_record("https://example.com/2");

    // Set both records
    cache.set_url(&code1, &record1).await.unwrap();
    cache.set_url(&code2, &record2).await.unwrap();

    // Verify both are retrievable
    let result1 = cache.get_url(&code1).await.unwrap();
    let result2 = cache.get_url(&code2).await.unwrap();

    assert_eq!(result1.unwrap().original_url, "https://example.com/1");
    assert_eq!(result2.unwrap().original_url, "https://example.com/2");

    // Delete one and verify the other still exists
    cache.del(&code1).await.unwrap();

    let result1 = cache.get_url(&code1).await.unwrap();
    let result2 = cache.get_url(&code2).await.unwrap();

    assert!(result1.is_none());
    assert!(result2.is_some());
}

#[tokio::test]
async fn test_redis_cache_custom_prefix() {
    let fixture = RedisTestContainer::start().await;
    let conn1 = fixture.create_connection().await;
    let conn2 = fixture.create_connection().await;

    let cache1 = RedisUrlCache::with_prefix(conn1, "prefix1:");
    let cache2 = RedisUrlCache::with_prefix(conn2, "prefix2:");

    let code = ShortCode::new("prefix_test").unwrap();
    let record = create_test_record("https://example.com/prefix");

    // Set in cache1 only
    cache1.set_url(&code, &record).await.unwrap();

    // Should be found in cache1
    let result1 = cache1.get_url(&code).await.unwrap();
    assert!(result1.is_some());

    // Should not be found in cache2 (different prefix)
    let result2 = cache2.get_url(&code).await.unwrap();
    assert!(result2.is_none(), "Different prefix should isolate caches");
}

#[tokio::test]
async fn test_redis_cache_overwrite() {
    let fixture = RedisTestContainer::start().await;
    let conn = fixture.create_connection().await;
    let cache = RedisUrlCache::new(conn);

    let code = ShortCode::new("overwrite").unwrap();
    let record1 = create_test_record("https://example.com/old");
    let record2 = create_test_record("https://example.com/new");

    // Set initial record
    cache.set_url(&code, &record1).await.unwrap();
    let result = cache.get_url(&code).await.unwrap();
    assert_eq!(result.unwrap().original_url, "https://example.com/old");

    // Overwrite with new record
    cache.set_url(&code, &record2).await.unwrap();
    let result = cache.get_url(&code).await.unwrap();
    assert_eq!(result.unwrap().original_url, "https://example.com/new");
}

#[tokio::test]
async fn test_redis_cache_nonexistent_key() {
    let fixture = RedisTestContainer::start().await;
    let conn = fixture.create_connection().await;
    let cache = RedisUrlCache::new(conn);

    let code = ShortCode::new("nonexistent").unwrap();

    // Try to get a key that doesn't exist
    let result = cache.get_url(&code).await.unwrap();
    assert!(result.is_none());

    // Try to delete a key that doesn't exist (should not error)
    let result = cache.del(&code).await;
    assert!(result.is_ok(), "Deleting nonexistent key should not error");
}

#[tokio::test]
async fn test_redis_cache_with_ttl_via_redis() {
    let fixture = RedisTestContainer::start().await;
    let conn = fixture.create_connection().await;

    // Manually set a key with TTL using raw Redis commands
    let mut redis_conn = fixture.create_connection().await;
    let code = "ttl_test";
    let key = format!("wh:url:{}", code);
    let record = create_test_record("https://example.com/ttl");

    redis_conn
        .set_ex::<_, _, ()>(&key, serde_json::to_string(&record).unwrap(), 1)
        .await
        .unwrap();

    // Create cache and verify it can read the key
    let cache = RedisUrlCache::new(conn);
    let code = ShortCode::new("ttl_test").unwrap();

    let result = cache.get_url(&code).await.unwrap();
    assert!(result.is_some(), "Should be able to read key with TTL");

    // Wait for TTL to expire
    tokio::time::sleep(Duration::from_secs(2)).await;

    let result = cache.get_url(&code).await.unwrap();
    assert!(result.is_none(), "Key should be expired after TTL");
}
