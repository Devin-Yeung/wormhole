use crate::error::Result;
use crate::shortcode::ShortCode;
use async_trait::async_trait;
use jiff::{SignedDuration, Timestamp};
use serde::{Deserialize, Serialize};

/// Expiration policy for a shortened URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExpirationPolicy {
    /// The shortened URL never expires.
    Never,
    /// The shortened URL expires after a certain duration.
    AfterDuration(SignedDuration),
    /// The shortened URL expires at a specific timestamp.
    AtTimestamp(Timestamp),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Shortens a given URL based on the provided parameters and returns the generated short code.
    async fn shorten(&self, params: ShortenParams) -> Result<ShortCode>;

    /// Retrieves the original URL associated with the given short code.
    async fn get(&self, id: &ShortCode) -> Result<Option<String>>;
}
