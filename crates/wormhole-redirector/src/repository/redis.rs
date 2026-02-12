use async_trait::async_trait;
use redis::AsyncCommands;
use std::time::Duration;
use tracing::{debug, trace, warn};
use wormhole_core::{ReadRepository, Result, ShortCode, UrlRecord};

/// Generates a cache key for a short code.
fn cache_key(code: &ShortCode) -> String {
    format!("wh:url:{}", code.as_str())
}

/// A read-only repository decorator that adds Redis caching.
///
/// Read operations check Redis first, falling back to the inner repository.
/// Successful reads from the inner repository are cached.
#[derive(Debug, Clone)]
pub struct CachedRepository<R> {
    inner: R,
    redis: redis::aio::MultiplexedConnection,
    default_ttl: Option<Duration>,
}

impl<R: ReadRepository> CachedRepository<R> {
    /// Creates a new cached repository decorator.
    ///
    /// # Arguments
    ///
    /// * `inner` - The underlying read-only repository implementation
    /// * `redis` - A multiplexed Redis connection
    /// * `default_ttl` - Optional default TTL for cached entries
    pub fn new(
        inner: R,
        redis: redis::aio::MultiplexedConnection,
        default_ttl: Option<Duration>,
    ) -> Self {
        Self {
            inner,
            redis,
            default_ttl,
        }
    }
}

#[async_trait]
impl<R: ReadRepository> ReadRepository for CachedRepository<R> {
    async fn get(&self, code: &ShortCode) -> Result<Option<UrlRecord>> {
        let key = cache_key(code);
        trace!(code = %code, "Fetching URL record from cache");

        // 1. Try Redis first
        let mut redis = self.redis.clone();
        match redis.get::<_, Option<String>>(&key).await {
            Ok(Some(cached)) => {
                debug!(code = %code, "Cache hit for short code");
                match serde_json::from_str::<UrlRecord>(&cached) {
                    Ok(record) => return Ok(Some(record)),
                    Err(e) => {
                        warn!(code = %code, error = %e, "Failed to deserialize cached record");
                        // Continue to fetch from inner repository
                    }
                }
            }
            Ok(None) => {
                trace!(code = %code, "Cache miss for short code");
            }
            Err(e) => {
                warn!(code = %code, error = %e, "Redis error on get, falling back to inner repository");
            }
        }

        // 2. On miss, call inner.get()
        trace!(code = %code, "Fetching from inner repository");
        let result = self.inner.get(code).await?;

        // 3. Cache result if found
        if let Some(ref record) = result {
            match serde_json::to_string(record) {
                Ok(json) => {
                    let cache_result = if let Some(ttl) = self.default_ttl {
                        redis.set_ex::<_, _, ()>(&key, json, ttl.as_secs()).await
                    } else {
                        redis.set::<_, _, ()>(&key, json).await
                    };

                    if let Err(e) = cache_result {
                        warn!(code = %code, error = %e, "Failed to cache record in Redis");
                    } else {
                        debug!(code = %code, "Cached record in Redis");
                    }
                }
                Err(e) => {
                    warn!(code = %code, error = %e, "Failed to serialize record for caching");
                }
            }
        }

        Ok(result)
    }

    async fn exists(&self, code: &ShortCode) -> Result<bool> {
        let key = cache_key(code);
        trace!(code = %code, "Checking existence in cache");

        // Check cache first
        let mut redis = self.redis.clone();
        match redis.exists::<_, bool>(&key).await {
            Ok(true) => {
                debug!(code = %code, "Cache indicates code exists");
                return Ok(true);
            }
            Ok(false) => {
                trace!(code = %code, "Cache miss for existence check");
            }
            Err(e) => {
                warn!(code = %code, error = %e, "Redis error on exists check, falling back to inner repository");
            }
        }

        // Fall back to inner repository
        self.inner.exists(code).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn code(s: &str) -> ShortCode {
        ShortCode::new_unchecked(s)
    }

    // Note: These tests would require a running Redis instance.
    // For unit tests without Redis, we would need to mock the connection.
    // The following tests document the expected behavior:

    #[test]
    fn cache_key_format() {
        let c = code("abc123");
        assert_eq!(cache_key(&c), "wh:url:abc123");
    }
}
