use crate::error::Result;
use crate::repository::Repository;
use async_trait::async_trait;
use dashmap::DashMap;
use jiff::Timestamp;

/// In-memory storage entry for a URL mapping.
#[derive(Debug, Clone)]
struct Entry {
    original_url: String,
    expire_at: Option<Timestamp>,
}

/// In-memory implementation of the Repository trait using DashMap.
///
/// DashMap provides better concurrency than RwLock<HashMap> because it
/// uses sharded locks, allowing concurrent reads and writes to different
/// buckets without blocking.
#[derive(Debug, Clone)]
pub struct InMemoryRepository {
    storage: DashMap<u64, Entry>,
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
    async fn save(&self, id: u64, original_url: &str, expire_at: Option<Timestamp>) -> Result<()> {
        let entry = Entry {
            original_url: original_url.to_string(),
            expire_at,
        };
        self.storage.insert(id, entry);
        Ok(())
    }

    async fn get(&self, id: u64) -> Result<Option<String>> {
        // Check if the entry exists
        if let Some(entry) = self.storage.get(&id) {
            // Check if expired
            if let Some(expire_at) = entry.expire_at {
                if Timestamp::now() >= expire_at {
                    drop(entry); // Release the read lock
                    self.storage.remove(&id);
                    return Ok(None);
                }
            }
            return Ok(Some(entry.original_url.clone()));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::SignedDuration;

    #[tokio::test]
    async fn test_save_and_get() {
        let repo = InMemoryRepository::new();

        repo.save(1, "https://example.com", None).await.unwrap();

        let result = repo.get(1).await.unwrap();
        assert_eq!(result, Some("https://example.com".to_string()));
    }

    #[tokio::test]
    async fn test_get_nonexistent() {
        let repo = InMemoryRepository::new();

        let result = repo.get(999).await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_expired_entry() {
        let repo = InMemoryRepository::new();

        // Create an entry that expired 1 second ago
        let expire_at = Timestamp::now() - SignedDuration::from_secs(1);
        repo.save(1, "https://example.com", Some(expire_at))
            .await
            .unwrap();

        // Should return None for expired entry
        let result = repo.get(1).await.unwrap();
        assert_eq!(result, None);

        // Entry should be removed from storage
        assert!(!repo.storage.contains_key(&1));
    }

    #[tokio::test]
    async fn test_not_expired_entry() {
        let repo = InMemoryRepository::new();

        // Create an entry that expires 1 hour from now
        let expire_at = Timestamp::now() + SignedDuration::from_hours(1);
        repo.save(1, "https://example.com", Some(expire_at))
            .await
            .unwrap();

        let result = repo.get(1).await.unwrap();
        assert_eq!(result, Some("https://example.com".to_string()));
    }

    #[tokio::test]
    async fn test_update_existing_entry() {
        let repo = InMemoryRepository::new();

        repo.save(1, "https://example.com", None).await.unwrap();
        repo.save(1, "https://updated.com", None).await.unwrap();

        let result = repo.get(1).await.unwrap();
        assert_eq!(result, Some("https://updated.com".to_string()));
    }

    #[tokio::test]
    async fn test_with_capacity() {
        let repo = InMemoryRepository::with_capacity(100);

        repo.save(1, "https://example.com", None).await.unwrap();

        let result = repo.get(1).await.unwrap();
        assert_eq!(result, Some("https://example.com".to_string()));
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        use std::sync::Arc;

        let repo = Arc::new(InMemoryRepository::new());
        let mut handles = vec![];

        // Spawn multiple tasks that write concurrently
        for i in 0..10 {
            let repo = Arc::clone(&repo);
            let handle = tokio::spawn(async move {
                repo.save(i, &format!("https://example{}.com", i), None)
                    .await
                    .unwrap();
            });
            handles.push(handle);
        }

        // Spawn multiple tasks that read concurrently
        for i in 0..10 {
            let repo = Arc::clone(&repo);
            let handle = tokio::spawn(async move {
                // May return None if the write hasn't happened yet, that's ok
                let _ = repo.get(i).await;
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        // Verify all writes succeeded
        for i in 0..10 {
            let result = repo.get(i).await.unwrap();
            assert_eq!(result, Some(format!("https://example{}.com", i)));
        }
    }
}
