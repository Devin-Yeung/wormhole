//! URL shortener service implementation.
//!
//! This crate provides the shortener service implementation and the
//! code generator trait. Core types are re-exported from `wormhole_core`.

pub mod generator;
pub mod service;

// Re-export core types
pub use wormhole_core::{
    error, shortcode, shortener, CacheError, ExpirationPolicy, ReadRepository, Repository,
    ShortCode, ShortenParams, Shortener, ShortenerError, StorageError, UrlRecord,
};

// Re-export InMemoryRepository from storage
pub use wormhole_storage::InMemoryRepository;
