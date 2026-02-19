use async_trait::async_trait;
use std::future::Future;
use wormhole_core::{CacheError, ShortCode, UrlRecord};

pub type Result<T> = std::result::Result<T, CacheError>;

/// A cache for URL records.
///
/// This trait provides a domain-specific caching abstraction for [`UrlRecord`]s,
/// using [`ShortCode`] as the key.
#[async_trait]
pub trait UrlCache: Send + Sync + 'static {
    /// Get URL record from cache.
    ///
    /// Returns `Ok(None)` if the key is not in the cache.
    async fn get_url(&self, code: &ShortCode) -> Result<Option<UrlRecord>>;

    /// Store URL record in cache.
    async fn set_url(&self, code: &ShortCode, record: &UrlRecord) -> Result<()>;

    /// Remove URL record from cache.
    async fn del(&self, code: &ShortCode) -> Result<()>;

    /// Get URL record from cache, computing it if not present.
    async fn get_or_compute<F, Fut>(&self, code: &ShortCode, fetch: F) -> Result<Option<UrlRecord>>
    where
        F: FnOnce(&ShortCode) -> Fut + Send,
        Fut: Future<Output = Result<Option<UrlRecord>>> + Send,
    {
        match self.get_url(code).await? {
            Some(record) => Ok(Some(record)),
            None => {
                let record = fetch(code).await?;
                if let Some(ref value) = record {
                    self.set_url(code, value).await?;
                }
                Ok(record)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[derive(Default)]
    struct TestCache {
        items: Mutex<HashMap<String, UrlRecord>>,
    }

    #[async_trait]
    impl UrlCache for TestCache {
        async fn get_url(&self, code: &ShortCode) -> Result<Option<UrlRecord>> {
            let items = self.items.lock().await;
            Ok(items.get(code.as_str()).cloned())
        }

        async fn set_url(&self, code: &ShortCode, record: &UrlRecord) -> Result<()> {
            let mut items = self.items.lock().await;
            items.insert(code.as_str().to_string(), record.clone());
            Ok(())
        }

        async fn del(&self, code: &ShortCode) -> Result<()> {
            let mut items = self.items.lock().await;
            items.remove(code.as_str());
            Ok(())
        }
    }

    fn test_record(url: &str) -> UrlRecord {
        UrlRecord {
            original_url: url.to_string(),
            expire_at: None,
        }
    }

    #[tokio::test]
    async fn get_or_compute_returns_cached_value_without_fetch() {
        let cache = TestCache::default();
        let code = ShortCode::new_unchecked("abc123");
        let existing = test_record("https://cached.example");
        cache.set_url(&code, &existing).await.unwrap();

        let fetch_calls = Arc::new(AtomicUsize::new(0));
        let result = cache
            .get_or_compute(&code, {
                let fetch_calls = Arc::clone(&fetch_calls);
                move |_| async move {
                    fetch_calls.fetch_add(1, Ordering::SeqCst);
                    Ok(Some(test_record("https://fetched.example")))
                }
            })
            .await
            .unwrap();

        assert_eq!(result, Some(existing));
        assert_eq!(fetch_calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn get_or_compute_fetches_and_backfills_on_cache_miss() {
        let cache = TestCache::default();
        let code = ShortCode::new_unchecked("miss123");
        let fetched = test_record("https://fetched.example");

        let result = cache
            .get_or_compute(&code, |_code| async { Ok(Some(fetched.clone())) })
            .await
            .unwrap();

        assert_eq!(result, Some(fetched.clone()));
        assert_eq!(cache.get_url(&code).await.unwrap(), Some(fetched));
    }
}
