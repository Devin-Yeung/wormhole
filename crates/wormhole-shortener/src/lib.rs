//! URL shortener service implementation.
//!
//! This crate provides the shortener service implementation and the
//! code generator trait. Core types are re-exported from `wormhole_core`.

pub mod error;
pub mod grpc;
pub mod service;
pub mod shortener;

pub use error::ShortenerError;
