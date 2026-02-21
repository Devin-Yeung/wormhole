//! Redirector service library with caching support.
//!
//! This crate provides a [`RedirectorService`] that can resolve short codes
//! to their original URLs. It uses the Repository decorator pattern to
//! add transparent caching via either Redis or in-memory (Moka) caches.

mod error;
pub mod grpc;
pub mod redirector;
pub mod repository;
pub mod service;

pub use error::{RedirectorError, Result};
pub use repository::CachedRepository;
pub use service::RedirectorService;
