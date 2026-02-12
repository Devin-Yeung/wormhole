pub mod memory;

use crate::error::Result;
use crate::shortcode::ShortCode;
use async_trait::async_trait;
use jiff::Timestamp;
use serde::{Deserialize, Serialize};

/// A stored URL record in the repository.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UrlRecord {
    /// The original URL that was shortened.
    pub original_url: String,
    /// When the record expires, if ever.
    pub expire_at: Option<Timestamp>,
}

/// A read-only view of a repository.
///
/// This trait provides only the read operations from [`Repository`],
/// allowing services like the redirector to have read-only access.
#[async_trait]
pub trait ReadRepository: Send + Sync + 'static {
    /// Retrieves the URL record for a given short code.
    /// Returns `None` if the code does not exist.
    async fn get(&self, code: &ShortCode) -> Result<Option<UrlRecord>>;

    /// Checks whether a short code already exists in the repository.
    async fn exists(&self, code: &ShortCode) -> Result<bool>;
}

#[async_trait]
pub trait Repository: ReadRepository {
    /// Inserts a new URL record. Returns `Err(AliasConflict)` if the code already exists.
    async fn insert(&self, code: &ShortCode, record: UrlRecord) -> Result<()>;

    /// Deletes the URL record for a given short code.
    /// Returns `true` if the record existed and was removed.
    async fn delete(&self, code: &ShortCode) -> Result<bool>;
}
