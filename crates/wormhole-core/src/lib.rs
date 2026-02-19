//! Core types and traits for the Wormhole URL shortener.
//!
//! This crate provides shared types and traits used by both the
//! shortener service and the redirector service.

pub mod base58;
pub mod error;
pub mod shortcode;
pub mod slim_id;

pub use error::{CacheError, CoreError};
pub use shortcode::{ShortCode, UrlRecord};
