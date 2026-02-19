//! URL shortener service implementation.
//!
//! This crate provides the shortener service implementation and the
//! code generator trait. Core types are re-exported from `wormhole_core`.

pub mod error;
pub mod generator;
pub mod service;
pub mod shortener;

pub use error::ShortenerError;
pub use shortener::{ExpirationPolicy, ShortenParams, Shortener};
pub use wormhole_core::{base58, shortcode, slim_id, CacheError, CoreError, ShortCode, UrlRecord};
pub use wormhole_storage::{ReadRepository, Repository, StorageError};

// Re-export InMemoryRepository from storage
pub use wormhole_storage::InMemoryRepository;
