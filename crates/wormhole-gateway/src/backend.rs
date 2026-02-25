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
