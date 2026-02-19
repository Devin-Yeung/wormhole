use async_trait::async_trait;
use redis::AsyncCommands;
use tracing::{debug, trace, warn};
use wormhole_core::{CacheError, ShortCode, UrlRecord};

use crate::UrlCache;

/// Type alias for cache results.
pub type Result<T> = std::result::Result<T, CacheError>;

/// A Redis-based implementation of [`UrlCache`].
///
/// This implementation stores URL records as JSON strings in Redis,
/// using a configurable key prefix.
#[derive(Debug, Clone)]
pub struct RedisUrlCache {
    conn: redis::aio::MultiplexedConnection,
    key_prefix: String,
}

fn map_redis_error(operation: &str, err: redis::RedisError) -> CacheError {
    let message = format!("{operation}: {err}");
    if message.to_ascii_lowercase().contains("timed out") {
        CacheError::Timeout(message)
    } else {
        CacheError::Operation(message)
    }
}

impl RedisUrlCache {
    /// Creates a new Redis URL cache.
    ///
    /// # Arguments
    ///
    /// * `conn` - A multiplexed Redis connection
    pub fn new(conn: redis::aio::MultiplexedConnection) -> Self {
        Self {
            conn,
            key_prefix: "wh:url:".to_string(),
        }
    }

    /// Creates a new Redis URL cache with a custom key prefix.
    ///
    /// # Arguments
    ///
    /// * `conn` - A multiplexed Redis connection
    /// * `key_prefix` - Custom prefix for cache keys (e.g., "myapp:url:")
    pub fn with_prefix(
        conn: redis::aio::MultiplexedConnection,
        key_prefix: impl Into<String>,
    ) -> Self {
        Self {
            conn,
            key_prefix: key_prefix.into(),
        }
    }

    /// Generates the cache key for a short code.
    fn cache_key(&self, code: &ShortCode) -> String {
        format!("{}{}", self.key_prefix, code.as_str())
    }
}

#[async_trait]
impl UrlCache for RedisUrlCache {
    async fn get_url(&self, code: &ShortCode) -> Result<Option<UrlRecord>> {
        let key = self.cache_key(code);
        trace!(code = %code, "Fetching URL record from Redis cache");

        let mut conn = self.conn.clone();
        match conn.get::<_, Option<String>>(&key).await {
            Ok(Some(cached)) => {
                debug!(code = %code, "Cache hit in Redis");
                match serde_json::from_str::<UrlRecord>(&cached) {
                    Ok(record) => Ok(Some(record)),
                    Err(e) => {
                        warn!(code = %code, error = %e, "Failed to deserialize cached record");
                        Err(CacheError::InvalidData(format!(
                            "invalid cached value for key '{key}': {e}"
                        )))
                    }
                }
            }
            Ok(None) => {
                trace!(code = %code, "Cache miss in Redis");
                Ok(None)
            }
            Err(e) => {
                warn!(code = %code, error = %e, "Redis error on get");
                Err(map_redis_error("failed to fetch value from Redis", e))
            }
        }
    }

    async fn set_url(&self, code: &ShortCode, record: &UrlRecord) -> Result<()> {
        let key = self.cache_key(code);
        trace!(code = %code, "Storing URL record in Redis cache");

        let json = match serde_json::to_string(record) {
            Ok(json) => json,
            Err(e) => {
                warn!(code = %code, error = %e, "Failed to serialize record for caching");
                return Err(CacheError::Serialization(format!(
                    "failed to serialize cache value: {e}"
                )));
            }
        };

        let mut conn = self.conn.clone();
        match conn.set::<_, _, ()>(&key, json).await {
            Ok(()) => {
                debug!(code = %code, "Cached record in Redis");
                Ok(())
            }
            Err(e) => {
                warn!(code = %code, error = %e, "Failed to cache record in Redis");
                Err(map_redis_error("failed to write value to Redis", e))
            }
        }
    }

    async fn del(&self, code: &ShortCode) -> Result<()> {
        let key = self.cache_key(code);
        trace!(code = %code, "Removing URL record from Redis cache");

        let mut conn = self.conn.clone();
        match conn.del::<_, ()>(&key).await {
            Ok(()) => {
                debug!(code = %code, "Removed record from Redis cache");
                Ok(())
            }
            Err(e) => {
                warn!(code = %code, error = %e, "Failed to remove record from Redis cache");
                Err(map_redis_error("failed to delete value from Redis", e))
            }
        }
    }
}
