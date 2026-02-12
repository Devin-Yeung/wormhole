//! Core types and traits for the Wormhole URL shortener.
//!
//! This crate provides shared types and traits used by both the
//! shortener service and the redirector service.

pub mod error;
pub mod repository;
pub mod shortcode;
pub mod shortener;

pub use error::{Error, Result};
pub use repository::{memory::InMemoryRepository, ReadRepository, Repository, UrlRecord};
pub use shortcode::ShortCode;
pub use shortener::{ExpirationPolicy, ShortenParams, Shortener};
