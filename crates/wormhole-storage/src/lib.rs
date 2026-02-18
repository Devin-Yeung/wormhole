pub mod error;
pub mod memory;
pub mod mysql;

pub use error::{Result, StorageError};
pub use memory::InMemoryRepository;
pub use mysql::MySqlRepository;

use async_trait::async_trait;
use wormhole_core::{ShortCode, UrlRecord};

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
