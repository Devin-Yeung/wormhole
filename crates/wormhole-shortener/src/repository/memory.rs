use crate::error::{Error, Result};
use crate::repository::{Repository, UrlRecord};
use crate::shortcode::ShortCode;
use async_trait::async_trait;
use dashmap::DashMap;
use jiff::Timestamp;

/// In-memory storage entry for a URL mapping.
#[derive(Debug, Clone)]
struct Entry {
    original_url: String,
    expire_at: Option<Timestamp>,
}

impl Entry {
    fn is_expired(&self) -> bool {
        self.expire_at
            .is_some_and(|expire_at| Timestamp::now() >= expire_at)
    }

    fn into_record(self) -> UrlRecord {
        UrlRecord {
            original_url: self.original_url,
            expire_at: self.expire_at,
        }
    }
}

/// In-memory implementation of the Repository trait using DashMap.
///
/// DashMap provides better concurrency than RwLock<HashMap> because it
/// uses sharded locks, allowing concurrent reads and writes to different
/// buckets without blocking.
#[derive(Debug, Clone)]
pub struct InMemoryRepository {
    storage: DashMap<String, Entry>,
}

impl InMemoryRepository {
    /// Creates a new in-memory repository.
    pub fn new() -> Self {
        Self {
            storage: DashMap::new(),
        }
    }

    /// Creates a new in-memory repository with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            storage: DashMap::with_capacity(capacity),
        }
    }
}

impl Default for InMemoryRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Repository for InMemoryRepository {
    async fn insert(&self, code: &ShortCode, record: UrlRecord) -> Result<()> {
        let key = code.as_str().to_owned();
        let entry = Entry {
            original_url: record.original_url,
            expire_at: record.expire_at,
        };

        // Check-and-insert: reject if the code is already taken (and not expired).
        let existing = self.storage.get(&key);
        if let Some(ref e) = existing {
            if !e.is_expired() {
                return Err(Error::AliasConflict(code.to_string()));
            }
            // Expired entry — drop the read guard, then remove & re-insert below.
            drop(existing);
        }

        self.storage.insert(key, entry);
        Ok(())
    }

    async fn get(&self, code: &ShortCode) -> Result<Option<UrlRecord>> {
        let key = code.as_str();

        let Some(entry) = self.storage.get(key) else {
            return Ok(None);
        };

        if entry.is_expired() {
            drop(entry);
            self.storage.remove(key);
            return Ok(None);
        }

        Ok(Some(entry.clone().into_record()))
    }

    async fn delete(&self, code: &ShortCode) -> Result<bool> {
        Ok(self.storage.remove(code.as_str()).is_some())
    }

    async fn exists(&self, code: &ShortCode) -> Result<bool> {
        let key = code.as_str();

        let Some(entry) = self.storage.get(key) else {
            return Ok(false);
        };

        if entry.is_expired() {
            drop(entry);
            self.storage.remove(key);
            return Ok(false);
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::SignedDuration;

    fn code(s: &str) -> ShortCode {
        ShortCode::new_unchecked(s)
    }

    fn record(url: &str, expire_at: Option<Timestamp>) -> UrlRecord {
        UrlRecord {
            original_url: url.to_string(),
            expire_at,
        }
    }

    #[tokio::test]
    async fn save_and_get() {
        let repo = InMemoryRepository::new();

        repo.insert(&code("abc123"), record("https://example.com", None))
            .await
            .unwrap();

        let result = repo.get(&code("abc123")).await.unwrap().unwrap();
        assert_eq!(result.original_url, "https://example.com");
        assert_eq!(result.expire_at, None);
    }

    #[tokio::test]
    async fn get_nonexistent() {
        let repo = InMemoryRepository::new();

        let result = repo.get(&code("nope")).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn insert_conflict() {
        let repo = InMemoryRepository::new();

        repo.insert(&code("abc123"), record("https://example.com", None))
            .await
            .unwrap();

        let err = repo
            .insert(&code("abc123"), record("https://other.com", None))
            .await
            .unwrap_err();

        assert!(matches!(err, Error::AliasConflict(_)));
    }

    #[tokio::test]
    async fn insert_over_expired_entry() {
        let repo = InMemoryRepository::new();
        let expired = Timestamp::now() - SignedDuration::from_secs(1);

        repo.insert(&code("abc123"), record("https://old.com", Some(expired)))
            .await
            .unwrap();

        // Should succeed because the existing entry is expired.
        repo.insert(&code("abc123"), record("https://new.com", None))
            .await
            .unwrap();

        let result = repo.get(&code("abc123")).await.unwrap().unwrap();
        assert_eq!(result.original_url, "https://new.com");
    }

    #[tokio::test]
    async fn expired_entry_returns_none() {
        let repo = InMemoryRepository::new();
        let expired = Timestamp::now() - SignedDuration::from_secs(1);

        repo.insert(
            &code("abc123"),
            record("https://example.com", Some(expired)),
        )
        .await
        .unwrap();

        let result = repo.get(&code("abc123")).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn not_expired_entry() {
        let repo = InMemoryRepository::new();
        let future = Timestamp::now() + SignedDuration::from_hours(1);

        repo.insert(&code("abc123"), record("https://example.com", Some(future)))
            .await
            .unwrap();

        let result = repo.get(&code("abc123")).await.unwrap().unwrap();
        assert_eq!(result.original_url, "https://example.com");
    }

    #[tokio::test]
    async fn delete_existing() {
        let repo = InMemoryRepository::new();

        repo.insert(&code("abc123"), record("https://example.com", None))
            .await
            .unwrap();

        assert!(repo.delete(&code("abc123")).await.unwrap());
        assert!(repo.get(&code("abc123")).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn delete_nonexistent() {
        let repo = InMemoryRepository::new();

        assert!(!repo.delete(&code("nope")).await.unwrap());
    }

    #[tokio::test]
    async fn exists_checks() {
        let repo = InMemoryRepository::new();

        assert!(!repo.exists(&code("abc123")).await.unwrap());

        repo.insert(&code("abc123"), record("https://example.com", None))
            .await
            .unwrap();

        assert!(repo.exists(&code("abc123")).await.unwrap());
    }

    #[tokio::test]
    async fn exists_returns_false_for_expired() {
        let repo = InMemoryRepository::new();
        let expired = Timestamp::now() - SignedDuration::from_secs(1);

        repo.insert(
            &code("abc123"),
            record("https://example.com", Some(expired)),
        )
        .await
        .unwrap();

        assert!(!repo.exists(&code("abc123")).await.unwrap());
    }

    #[tokio::test]
    async fn concurrent_access() {
        use std::sync::Arc;

        let repo = Arc::new(InMemoryRepository::new());
        let mut handles = vec![];

        for i in 0..10u64 {
            let repo = Arc::clone(&repo);
            let handle = tokio::spawn(async move {
                let c = ShortCode::new_unchecked(format!("code-{:03}", i));
                let r = UrlRecord {
                    original_url: format!("https://example{}.com", i),
                    expire_at: None,
                };
                repo.insert(&c, r).await.unwrap();
            });
            handles.push(handle);
        }

        for i in 0..10u64 {
            let repo = Arc::clone(&repo);
            let handle = tokio::spawn(async move {
                let c = ShortCode::new_unchecked(format!("code-{:03}", i));
                let _ = repo.get(&c).await;
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        for i in 0..10u64 {
            let c = ShortCode::new_unchecked(format!("code-{:03}", i));
            let result = repo.get(&c).await.unwrap().unwrap();
            assert_eq!(result.original_url, format!("https://example{}.com", i));
        }
    }
}
