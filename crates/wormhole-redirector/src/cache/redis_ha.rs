//! Redis HA (High Availability) cache implementation with sentinel support.
//!
//! This implementation provides read/write splitting between Redis master and replicas,
//! using Redis Sentinel for service discovery and failover.

use async_trait::async_trait;
use redis::sentinel::{SentinelClient, SentinelServerType};
use redis::AsyncCommands;
use tracing::{debug, trace, warn};
use wormhole_core::{Result, ShortCode, UrlCache, UrlRecord};

/// A Redis HA implementation of [`UrlCache`] with read/write splitting.
///
/// This implementation uses two connections:
/// - Master connection: for SET and DEL operations (writes)
/// - Replica connection: for GET operations (reads)
///
/// It uses Redis Sentinel for service discovery, allowing automatic failover
/// when the master or replicas change.
#[derive(Debug, Clone)]
pub struct RedisHAUrlCache {
    pub(crate) master_conn: redis::aio::MultiplexedConnection,
    pub(crate) replica_conn: redis::aio::MultiplexedConnection,
    key_prefix: String,
}

impl RedisHAUrlCache {
    /// Creates a new Redis HA URL cache with the given connections.
    ///
    /// # Arguments
    ///
    /// * `master_conn` - Connection to the Redis master (for SET, DEL)
    /// * `replica_conn` - Connection to the Redis replica (for GET)
    pub fn new(
        master_conn: redis::aio::MultiplexedConnection,
        replica_conn: redis::aio::MultiplexedConnection,
    ) -> Self {
        Self {
            master_conn,
            replica_conn,
            key_prefix: "wh:url:".to_string(),
        }
    }

    /// Creates a new Redis HA URL cache with a custom key prefix.
    ///
    /// # Arguments
    ///
    /// * `master_conn` - Connection to the Redis master (for SET, DEL)
    /// * `replica_conn` - Connection to the Redis replica (for GET)
    /// * `key_prefix` - Custom prefix for cache keys (e.g., "myapp:url:")
    pub fn with_prefix(
        master_conn: redis::aio::MultiplexedConnection,
        replica_conn: redis::aio::MultiplexedConnection,
        key_prefix: impl Into<String>,
    ) -> Self {
        Self {
            master_conn,
            replica_conn,
            key_prefix: key_prefix.into(),
        }
    }

    /// Creates a new Redis HA URL cache from sentinel addresses.
    ///
    /// This method connects to Redis Sentinel to resolve the master and replica
    /// addresses, enabling automatic failover.
    ///
    /// # Arguments
    ///
    /// * `sentinel_addrs` - List of sentinel addresses (host, port)
    /// * `service_name` - The sentinel service name (e.g., "mymaster")
    ///
    /// # Errors
    ///
    /// Returns an error if connection to sentinel fails or if master/replica
    /// cannot be resolved.
    pub async fn from_sentinel(
        sentinel_addrs: Vec<(&str, u16)>,
        service_name: &str,
    ) -> redis::RedisResult<Self> {
        // Build params for sentinel connections
        let sentinel_strs: Vec<String> = sentinel_addrs
            .iter()
            .map(|(host, port)| format!("redis://{}:{}", host, port))
            .collect();

        // Build sentinel client for master
        let mut sentinel_master = SentinelClient::build(
            sentinel_strs.clone(),
            service_name.to_string(),
            None,
            SentinelServerType::Master,
        )?;
        let master_client = sentinel_master.async_get_client().await?;
        let master_conn = master_client.get_multiplexed_async_connection().await?;

        // Build sentinel client for replica
        let mut sentinel_replica = SentinelClient::build(
            sentinel_strs,
            service_name.to_string(),
            None,
            SentinelServerType::Replica,
        )?;
        let replica_client = sentinel_replica.async_get_client().await?;
        let replica_conn = replica_client.get_multiplexed_async_connection().await?;

        Ok(Self::new(master_conn, replica_conn))
    }

    /// Creates a new Redis HA URL cache from sentinel addresses with custom key prefix.
    ///
    /// # Arguments
    ///
    /// * `sentinel_addrs` - List of sentinel addresses (host, port)
    /// * `service_name` - The sentinel service name (e.g., "mymaster")
    /// * `key_prefix` - Custom prefix for cache keys
    pub async fn from_sentinel_with_prefix(
        sentinel_addrs: Vec<(&str, u16)>,
        service_name: &str,
        key_prefix: impl Into<String>,
    ) -> redis::RedisResult<Self> {
        let prefix = key_prefix.into();
        let mut cache = Self::from_sentinel(sentinel_addrs, service_name).await?;
        cache.key_prefix = prefix;
        Ok(cache)
    }

    /// Generates the cache key for a short code.
    fn cache_key(&self, code: &ShortCode) -> String {
        format!("{}{}", self.key_prefix, code.as_str())
    }

    /// Returns a new cache with updated sentinel connections.
    ///
    /// This is useful for reconnecting after a failover.
    pub async fn reconnect(
        &self,
        sentinel_addrs: Vec<(&str, u16)>,
        service_name: &str,
    ) -> redis::RedisResult<Self> {
        Self::from_sentinel_with_prefix(sentinel_addrs, service_name, &self.key_prefix).await
    }
}

#[async_trait]
impl UrlCache for RedisHAUrlCache {
    async fn get_url(&self, code: &ShortCode) -> Result<Option<UrlRecord>> {
        let key = self.cache_key(code);
        trace!(code = %code, "Fetching URL record from Redis HA cache (replica)");

        let mut conn = self.replica_conn.clone();
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
                trace!(code = %code, "Cache miss in Redis HA (replica)");
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

        let mut conn = self.master_conn.clone();
        match conn.set::<_, _, ()>(&key, json).await {
            Ok(()) => {
                debug!(code = %code, "Cached record in Redis HA (master)");
                Ok(())
            }
            Err(e) => {
                warn!(code = %code, error = %e, "Failed to cache record in Redis HA (master)");
                Err(wormhole_core::Error::Storage(Box::new(e)))
            }
        }
    }

    async fn del(&self, code: &ShortCode) -> Result<()> {
        let key = self.cache_key(code);
        trace!(code = %code, "Removing URL record from Redis HA cache (master)");

        let mut conn = self.master_conn.clone();
        match conn.del::<_, ()>(&key).await {
            Ok(()) => {
                debug!(code = %code, "Removed record from Redis HA cache (master)");
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
    use super::*;

    fn test_code() -> ShortCode {
        ShortCode::new("test123").unwrap()
    }

    #[tokio::test]
    async fn test_cache_key_generation() {
        let sentinel_addrs = vec![("127.0.0.1", 26379)];
        // Skip if sentinel not available
        if RedisHAUrlCache::from_sentinel(sentinel_addrs.clone(), "mymaster")
            .await
            .is_err()
        {
            return;
        }

        let cache = RedisHAUrlCache::from_sentinel(sentinel_addrs, "mymaster")
            .await
            .unwrap();
        let code = test_code();
        assert_eq!(cache.cache_key(&code), "wh:url:test123");
    }
}
