use crate::repository::UrlRecord;
use crate::shortcode::ShortCode;
use async_trait::async_trait;
use jiff::Timestamp;
use std::time::Duration;

type Result<T> = std::result::Result<T, crate::error::ShortenerError>;

/// Expiration policy for a shortened URL.
#[derive(Debug, Clone)]
pub enum ExpirationPolicy {
    /// The shortened URL never expires.
    Never,
    /// The shortened URL expires after a certain duration from now.
    AfterDuration(Duration),
    /// The shortened URL expires at a specific timestamp.
    AtTimestamp(Timestamp),
}

/// Parameters for creating a shortened URL.
#[derive(Debug, Clone)]
pub struct ShortenParams {
    /// The original URL to be shortened.
    pub original_url: String,
    /// The expiration policy for the shortened URL.
    pub expiration: ExpirationPolicy,
    /// Optional custom alias for the shortened URL.
    pub custom_alias: Option<ShortCode>,
}

#[async_trait]
pub trait Shortener: Send + Sync + 'static {
    /// Creates a shortened URL and returns the generated short code.
    async fn shorten(&self, params: ShortenParams) -> Result<ShortCode>;

    /// Resolves a short code to its stored URL record.
    /// Returns `None` if the code does not exist or has expired.
    async fn resolve(&self, code: &ShortCode) -> Result<Option<UrlRecord>>;

    /// Deletes a shortened URL by its short code.
    /// Returns `true` if the record existed and was removed.
    async fn delete(&self, code: &ShortCode) -> Result<bool>;
}
