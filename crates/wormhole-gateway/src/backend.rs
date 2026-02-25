//! Gateway backend boundary traits and types.
//!
//! This module defines the **port layer** in our hexagonal architecture.
//! It exposes abstract traits (`UrlRead`, `UrlWrite`) that the application
//! layer depends on, keeping the handlers decoupled from concrete implementations
//! (local service, gRPC client, HTTP adapter, etc.).
//!
//! ## Design Rationale
//!
//! We separate read and write concerns into distinct traits because:
//! - Different adapters may have different capabilities (e.g., a read-through cache)
//! - It enables independent evolution of query and command paths
//! - The `UrlService` supertrait combines both when a full API surface is needed
//!
//! ## Command/Result DTOs
//!
//! Each operation uses command DTOs to:
//! - Provide a stable interface between layers
//! - Allow future extensibility without breaking existing implementations
//! - Keep internal adapter types from leaking into the application layer

mod error;
mod url_read;
mod url_write;

pub use crate::backend::url_read::{GetUrlCmd, GetUrlResult, UrlRead};
pub use crate::backend::url_write::{DeleteUrlCmd, UrlWrite, WriteUrlCmd, WriteUrlResult};

pub use error::{BackendError, Result};

/// Convenience port that represents the full URL API surface.
///
/// Callers can depend on a single trait object when they need both read and
/// write behavior, without coupling to any concrete implementation.
pub trait UrlService: UrlWrite + UrlRead {}

impl<T> UrlService for T where T: UrlWrite + UrlRead {}
