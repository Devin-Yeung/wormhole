use crate::generator::Generator;
use async_trait::async_trait;
use jiff::Timestamp;
use std::sync::Arc;
use wormhole_core::{
    ExpirationPolicy, Repository, ShortCode, ShortenParams, Shortener, ShortenerError, UrlRecord,
};

/// A concrete implementation of the `Shortener` trait.
///
/// This service wraps a `Repository` and a `Generator` to handle:
/// - Short code generation (auto-generated or custom)
/// - Expiration policy conversion
/// - URL validation
///
/// Note: The `Generator` implementation is responsible for ensuring
/// uniqueness of generated short codes. No collision retry is performed.
#[derive(Debug, Clone)]
pub struct ShortenerService<R, G> {
    repository: Arc<R>,
    generator: Arc<G>,
}

impl<R: Repository, G: Generator> ShortenerService<R, G> {
    /// Creates a new `ShortenerService` with a custom generator.
    pub fn new(repository: R, generator: G) -> Self {
        Self {
            repository: Arc::new(repository),
            generator: Arc::new(generator),
        }
    }

    /// Validates that the URL has a valid format (has a scheme and host).
    fn validate_url(url: &str) -> Result<(), ShortenerError> {
        if url.is_empty() {
            return Err(ShortenerError::InvalidUrl(
                "URL cannot be empty".to_string(),
            ));
        }

        // Basic validation: check for scheme and host presence
        // A valid URL should have "://" and something after it
        let parts: Vec<&str> = url.split("://").collect();
        if parts.len() < 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(ShortenerError::InvalidUrl(format!(
                "URL must have a valid scheme and host: {}",
                url
            )));
        }

        // Check for valid scheme (http or https)
        let scheme = parts[0].to_lowercase();
        if scheme != "http" && scheme != "https" {
            return Err(ShortenerError::InvalidUrl(format!(
                "URL scheme must be http or https: {}",
                scheme
            )));
        }

        Ok(())
    }

    /// Generates a short code using the configured generator.
    /// The generator is responsible for ensuring uniqueness.
    fn generate_code(&self) -> ShortCode {
        self.generator.generate()
    }
}

#[async_trait]
impl<R: Repository, G: Generator> Shortener for ShortenerService<R, G> {
    async fn shorten(&self, params: ShortenParams) -> Result<ShortCode, ShortenerError> {
        // Validate the URL
        Self::validate_url(&params.original_url)?;

        // Determine the short code to use
        let short_code = match params.custom_alias {
            Some(code) => {
                // Check for alias conflict
                if self
                    .repository
                    .exists(&code)
                    .await
                    .map_err(storage_to_shortener_error)?
                {
                    return Err(ShortenerError::AliasConflict(code.to_string()));
                }
                code
            }
            // the generator can always produce a new code, so no need to check for conflicts here
            None => self.generate_code(),
        };

        // Convert expiration policy to optional timestamp
        let expire_at = match params.expiration {
            ExpirationPolicy::Never => None,
            ExpirationPolicy::AfterDuration(duration) => {
                let future = Timestamp::now()
                    + jiff::SignedDuration::try_from(duration).map_err(|e| {
                        ShortenerError::InvalidUrl(format!("Invalid duration: {}", e))
                    })?;
                Some(future)
            }
            ExpirationPolicy::AtTimestamp(timestamp) => Some(timestamp),
        };

        // Create the URL record
        let record = UrlRecord {
            original_url: params.original_url,
            expire_at,
        };

        // Store in repository
        self.repository
            .insert(&short_code, record)
            .await
            .map_err(storage_to_shortener_error)?;

        Ok(short_code)
    }

    async fn resolve(&self, code: &ShortCode) -> Result<Option<UrlRecord>, ShortenerError> {
        self.repository
            .get(code)
            .await
            .map_err(storage_to_shortener_error)
    }

    async fn delete(&self, code: &ShortCode) -> Result<bool, ShortenerError> {
        self.repository
            .delete(code)
            .await
            .map_err(storage_to_shortener_error)
    }
}

/// Converts a StorageError to a ShortenerError.
fn storage_to_shortener_error(e: wormhole_core::StorageError) -> ShortenerError {
    match e {
        wormhole_core::StorageError::Conflict(code) => ShortenerError::AliasConflict(code),
        other => ShortenerError::Storage(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::seq::UniqueGenerator;
    use wormhole_storage::InMemoryRepository;

    fn test_service() -> ShortenerService<InMemoryRepository, UniqueGenerator> {
        let repo = InMemoryRepository::new();
        let generator = UniqueGenerator::with_prefix("wh");
        ShortenerService::new(repo, generator)
    }

    #[tokio::test]
    async fn shorten_with_auto_generated_code() {
        let service = test_service();

        let params = ShortenParams {
            original_url: "https://example.com".to_string(),
            expiration: ExpirationPolicy::Never,
            custom_alias: None,
        };

        let code = service.shorten(params).await.unwrap();
        assert_eq!(code.as_str().len(), 8); // "wh" + 6 digits
    }

    #[tokio::test]
    async fn shorten_with_custom_alias() {
        let service = test_service();

        let params = ShortenParams {
            original_url: "https://example.com".to_string(),
            expiration: ExpirationPolicy::Never,
            custom_alias: Some(ShortCode::new("my-alias").unwrap()),
        };

        let code = service.shorten(params).await.unwrap();
        assert_eq!(code.as_str(), "my-alias");
    }

    #[tokio::test]
    async fn shorten_with_duplicate_alias_fails() {
        let service = test_service();

        let params1 = ShortenParams {
            original_url: "https://example1.com".to_string(),
            expiration: ExpirationPolicy::Never,
            custom_alias: Some(ShortCode::new("my-alias").unwrap()),
        };

        let params2 = ShortenParams {
            original_url: "https://example2.com".to_string(),
            expiration: ExpirationPolicy::Never,
            custom_alias: Some(ShortCode::new("my-alias").unwrap()),
        };

        service.shorten(params1).await.unwrap();
        let err = service.shorten(params2).await.unwrap_err();
        assert!(matches!(err, ShortenerError::AliasConflict(_)));
    }

    #[tokio::test]
    async fn shorten_with_invalid_url_fails() {
        let service = test_service();

        let params = ShortenParams {
            original_url: "not-a-valid-url".to_string(),
            expiration: ExpirationPolicy::Never,
            custom_alias: None,
        };

        let err = service.shorten(params).await.unwrap_err();
        assert!(matches!(err, ShortenerError::InvalidUrl(_)));
    }

    #[tokio::test]
    async fn resolve_existing_url() {
        let service = test_service();

        let params = ShortenParams {
            original_url: "https://example.com".to_string(),
            expiration: ExpirationPolicy::Never,
            custom_alias: Some(ShortCode::new("abc123").unwrap()),
        };

        service.shorten(params).await.unwrap();

        let record = service
            .resolve(&ShortCode::new("abc123").unwrap())
            .await
            .unwrap();
        assert!(record.is_some());
        assert_eq!(record.unwrap().original_url, "https://example.com");
    }

    #[tokio::test]
    async fn resolve_nonexistent_url() {
        let service = test_service();

        let record = service
            .resolve(&ShortCode::new("nonexistent").unwrap())
            .await
            .unwrap();
        assert!(record.is_none());
    }

    #[tokio::test]
    async fn delete_existing_url() {
        let service = test_service();

        let params = ShortenParams {
            original_url: "https://example.com".to_string(),
            expiration: ExpirationPolicy::Never,
            custom_alias: Some(ShortCode::new("abc123").unwrap()),
        };

        service.shorten(params).await.unwrap();
        let deleted = service
            .delete(&ShortCode::new("abc123").unwrap())
            .await
            .unwrap();
        assert!(deleted);

        let record = service
            .resolve(&ShortCode::new("abc123").unwrap())
            .await
            .unwrap();
        assert!(record.is_none());
    }

    #[tokio::test]
    async fn delete_nonexistent_url() {
        let service = test_service();

        let deleted = service
            .delete(&ShortCode::new("nonexistent").unwrap())
            .await
            .unwrap();
        assert!(!deleted);
    }

    #[tokio::test]
    async fn shorten_with_custom_generator() {
        let service = test_service();

        let params = ShortenParams {
            original_url: "https://example.com".to_string(),
            expiration: ExpirationPolicy::Never,
            custom_alias: None,
        };

        let code1 = service.shorten(params.clone()).await.unwrap();
        let code2 = service.shorten(params.clone()).await.unwrap();

        assert_eq!(code1.as_str(), "wh000000");
        assert_eq!(code2.as_str(), "wh000001");
    }
}
