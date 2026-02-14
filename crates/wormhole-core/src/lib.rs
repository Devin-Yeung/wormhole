//! Core types and traits for the Wormhole URL shortener.
//!
//! This crate provides shared types and traits used by both the
//! shortener service and the redirector service.

pub mod cache;
pub mod error;
pub mod repository;
pub mod shortcode;
pub mod shortener;

pub use cache::UrlCache;
pub use error::{CacheError, Error, Result, ShortenerError, StorageError};
pub use repository::{ReadRepository, Repository, UrlRecord};
pub use shortcode::ShortCode;
pub use shortener::{ExpirationPolicy, ShortenParams, Shortener};
