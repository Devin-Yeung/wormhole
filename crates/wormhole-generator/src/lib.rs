pub mod seq;

use wormhole_core::ShortCode;

/// Trait for generating short codes.
///
/// Implementations are pure generators that don't interact with storage.
///
/// Implementations can vary from simple random generators to
/// distributed ID generators (e.g., Snowflake, UUID, etc.)
pub trait Generator: Send + Sync + 'static {
    type Output: Into<ShortCode>;
    /// Generates a type that can be converted into a globally unique short code.
    ///
    /// The generated code should be unique
    fn generate(&self) -> Self::Output;
}
