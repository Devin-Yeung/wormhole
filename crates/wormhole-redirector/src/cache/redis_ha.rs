use async_trait::async_trait;
use deadpool_redis::redis::AsyncCommands;
use tracing::{debug, trace, warn};
use wormhole_core::{Result, ShortCode, UrlCache, UrlRecord};

/// A Redis Sentinel-based high-availability implementation of [`UrlCache`].
///
/// This implementation uses separate connection pools for master (writes)
/// and replicas (reads), providing read scalability and automatic failover.
#[derive(Debug, Clone)]
pub struct RedisHAUrlCache {
    master_pool: deadpool_redis::sentinel::Pool,
    replica_pool: deadpool_redis::sentinel::Pool,
    key_prefix: String,
}

impl RedisHAUrlCache {
    /// Creates a new high-availability Redis URL cache using Sentinel.
    ///
    /// # Arguments
    ///
    /// * `sentinels` - List of sentinel addresses (e.g., `["redis://localhost:26379"]`)
    /// * `service_name` - The Redis service name to look up (e.g., "mymaster")
    pub fn new<T: AsRef<str>>(sentinels: Vec<T>, service_name: &str) -> Self {
        Self::with_prefix(sentinels, service_name, "wh:url:")
    }

    /// Creates a new HA Redis cache with a custom key prefix.
    ///
    /// # Arguments
    ///
    /// * `sentinels` - List of sentinel addresses
    /// * `service_name` - The Redis service name to look up
    /// * `key_prefix` - Custom prefix for cache keys (e.g., "myapp:url:")
    pub fn with_prefix<T: AsRef<str>>(
        sentinels: Vec<T>,
        service_name: &str,
        key_prefix: impl Into<String>,
    ) -> Self {
        let sentinels = sentinels
            .iter()
            .map(|s| s.as_ref().to_string())
            .collect::<Vec<_>>();

        let config = deadpool_redis::sentinel::Config::from_urls(
            sentinels.clone(),
            service_name.into(),
            deadpool_redis::sentinel::SentinelServerType::Master,
        );

        let master_pool = config.create_pool(None).unwrap();

        let replica_config = deadpool_redis::sentinel::Config::from_urls(
            sentinels,
            service_name.into(),
            deadpool_redis::sentinel::SentinelServerType::Replica,
        );

        let replica_pool = replica_config.create_pool(None).unwrap();

        Self {
            master_pool,
            replica_pool,
            key_prefix: key_prefix.into(),
        }
    }

    /// Generates the cache key for a short code.
    fn cache_key(&self, code: &ShortCode) -> String {
        format!("{}{}", self.key_prefix, code.as_str())
    }
}

#[async_trait]
impl UrlCache for RedisHAUrlCache {
    async fn get_url(&self, code: &ShortCode) -> Result<Option<UrlRecord>> {
        let key = self.cache_key(code);
        trace!(code = %code, "Fetching URL record from Redis HA cache (replica)");

        let mut conn = self.replica_pool.get().await.unwrap();

        match conn.get::<_, Option<String>>(&key).await {
            Ok(Some(cached)) => {
                debug!(code = %code, "Cache hit in Redis HA (replica)");
                match serde_json::from_str::<UrlRecord>(&cached) {
                    Ok(record) => Ok(Some(record)),
                    Err(e) => {
                        warn!(code = %code, error = %e, "Failed to deserialize cached record");
                        Ok(None)
                    }
                }
            }
            Ok(None) => {
                trace!(code = %code, "Cache miss in Redis HA");
                Ok(None)
            }
            Err(e) => {
                warn!(code = %code, error = %e, "Redis error on get from replica");
                Err(wormhole_core::Error::Storage(Box::new(e)))
            }
        }
    }

    async fn set_url(&self, code: &ShortCode, record: &UrlRecord) -> Result<()> {
        let key = self.cache_key(code);
        trace!(code = %code, "Storing URL record in Redis HA cache (master)");

        let json = match serde_json::to_string(record) {
            Ok(json) => json,
            Err(e) => {
                warn!(code = %code, error = %e, "Failed to serialize record for caching");
                return Err(wormhole_core::Error::Storage(Box::new(e)));
            }
        };

        let mut conn = match self.master_pool.get().await {
            Ok(conn) => conn,
            Err(e) => {
                warn!(code = %code, error = %e, "Failed to get connection from master pool");
                return Err(wormhole_core::Error::Storage(Box::new(e)));
            }
        };

        match conn.set::<_, _, ()>(&key, json).await {
            Ok(()) => {
                debug!(code = %code, "Cached record in Redis HA (master)");
                Ok(())
            }
            Err(e) => {
                warn!(code = %code, error = %e, "Failed to cache record in Redis HA");
                Err(wormhole_core::Error::Storage(Box::new(e)))
            }
        }
    }

    async fn del(&self, code: &ShortCode) -> Result<()> {
        let key = self.cache_key(code);
        trace!(code = %code, "Removing URL record from Redis HA cache (master)");

        let mut conn = match self.master_pool.get().await {
            Ok(conn) => conn,
            Err(e) => {
                warn!(code = %code, error = %e, "Failed to get connection from master pool");
                return Err(wormhole_core::Error::Storage(Box::new(e)));
            }
        };

        match conn.del::<_, ()>(&key).await {
            Ok(()) => {
                debug!(code = %code, "Removed record from Redis HA cache");
                Ok(())
            }
            Err(e) => {
                warn!(code = %code, error = %e, "Failed to remove record from Redis HA cache");
                Err(wormhole_core::Error::Storage(Box::new(e)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::cache::RedisHAUrlCache;
    use wormhole_test_infra::redis::{RedisHA, RedisHAConfig};

    #[tokio::test]
    async fn it_works() {
        let redis = RedisHA::new(RedisHAConfig::default()).await.unwrap();

        let sentinels = redis.sentinel_addresses().await;

        let _ = RedisHAUrlCache::new(sentinels, redis.name());
    }
}
