pub mod seq;

use crate::shortcode::ShortCode;

/// Trait for generating short codes.
///
/// Implementations are pure generators that don't interact with storage.
///
/// Implementations can vary from simple random generators to
/// distributed ID generators (e.g., Snowflake, UUID, etc.)
pub trait Generator: Send + Sync + 'static {
    /// Generates a short code.
    ///
    /// The generated code should be unique
    fn generate(&self) -> ShortCode;
}
