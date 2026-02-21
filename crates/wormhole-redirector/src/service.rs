use std::sync::Arc;

use crate::redirector::Redirector;
use async_trait::async_trait;
use jiff::Timestamp;
use tracing::{debug, trace};
use wormhole_core::{ShortCode, UrlRecord};
use wormhole_storage::ReadRepository;

/// Service for handling URL redirects.
///
/// Uses a read-only repository to fetch URL records and handles expiration checks.
#[derive(Debug, Clone)]
pub struct RedirectorService<R> {
    repository: Arc<R>,
}

impl<R: ReadRepository> RedirectorService<R> {
    /// Creates a new RedirectorService with the given repository.
    pub fn new(repository: R) -> Self {
        Self {
            repository: Arc::new(repository),
        }
    }

    /// Resolves a short code to its original URL.
    ///
    /// Returns `None` if the code doesn't exist or has expired.
    ///
    /// # Arguments
    ///
    /// * `code` - The short code to resolve
    ///
    /// # Returns
    ///
    /// * `Ok(Some(url))` - The original URL if found and not expired
    /// * `Ok(None)` - If the code doesn't exist or has expired
    /// * `Err(e)` - If there was an error accessing the repository
    pub async fn resolve(&self, code: &ShortCode) -> crate::Result<Option<UrlRecord>> {
        Redirector::resolve(self, code).await
    }
}

#[async_trait]
impl<R: ReadRepository> Redirector for RedirectorService<R> {
    async fn resolve(&self, code: &ShortCode) -> crate::Result<Option<UrlRecord>> {
        trace!(code = %code, "resolving short code");

        match self
            .repository
            .get(code)
            .await
            .map_err(crate::RedirectorError::from)?
        {
            Some(record) => {
                // Check expiration
                if let Some(expire_at) = record.expire_at {
                    if Timestamp::now() >= expire_at {
                        debug!(code = %code, "Record has expired");
                        return Ok(None);
                    }
                }

                debug!(code = %code, url = %record.original_url, "Resolved short code");
                Ok(Some(record))
            }
            None => {
                trace!(code = %code, "Short code not found");
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::SignedDuration;
    use wormhole_core::UrlRecord;
    use wormhole_storage::{InMemoryRepository, Repository};

    fn code(s: &str) -> ShortCode {
        ShortCode::new_unchecked(s)
    }

    fn record(url: &str, expire_at: Option<Timestamp>) -> UrlRecord {
        UrlRecord {
            original_url: url.to_string(),
            expire_at,
        }
    }

    async fn setup_with_record(
        code: &ShortCode,
        rec: UrlRecord,
    ) -> RedirectorService<InMemoryRepository> {
        let repo = InMemoryRepository::new();
        repo.insert(code, rec).await.unwrap();
        RedirectorService::new(repo)
    }

    #[tokio::test]
    async fn resolve_existing_code() {
        let c = code("abc123");
        let service = setup_with_record(&c, record("https://example.com", None)).await;

        let result = service.resolve(&c).await.unwrap();
        let result = result.expect("record should exist");
        assert_eq!(result.original_url, "https://example.com");
    }

    #[tokio::test]
    async fn resolve_nonexistent_code() {
        let service = RedirectorService::new(InMemoryRepository::new());
        let c = code("nope");

        let result = service.resolve(&c).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn resolve_expired_code() {
        let c = code("expired");
        let expired = Timestamp::now() - SignedDuration::from_secs(1);
        let service = setup_with_record(&c, record("https://example.com", Some(expired))).await;

        let result = service.resolve(&c).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn resolve_not_yet_expired() {
        let c = code("valid");
        let future = Timestamp::now() + SignedDuration::from_hours(1);
        let service = setup_with_record(&c, record("https://example.com", Some(future))).await;

        let result = service.resolve(&c).await.unwrap();
        let result = result.expect("record should exist");
        assert_eq!(result.original_url, "https://example.com");
    }
}
