//! URL shortener service implementation.
//!
//! This crate provides the shortener service implementation and the
//! code generator trait. Core types are re-exported from `wormhole_core`.

pub mod generator;
pub mod service;

// Re-export core types
pub use wormhole_core::{
    error, shortcode, shortener, Error, ExpirationPolicy, ReadRepository, Repository, Result,
    ShortCode, ShortenParams, Shortener, UrlRecord,
};

// Re-export InMemoryRepository from storage
pub use wormhole_storage::InMemoryRepository;
