pub mod seq;

use wormhole_core::ShortCode;
use wormhole_tinyflake::{Clock, Tinyflake};

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

impl<C: Clock + 'static> Generator for Tinyflake<C> {
    type Output = ShortCode;

    fn generate(&self) -> Self::Output {
        // `Generator` is intentionally infallible. Tinyflake errors indicate an
        // unrecoverable generator state (e.g., poisoned lock or epoch overflow).
        let id = self
            .next_id()
            .expect("tinyflake generator failed to produce the next id");
        ShortCode::generated(id)
    }
}

#[cfg(test)]
mod tests {
    use super::Generator;
    use jiff::Timestamp;
    use wormhole_core::ShortCode;
    use wormhole_tinyflake::{Tinyflake, TinyflakeSettings};

    #[test]
    fn tinyflake_implements_generator_trait() {
        let epoch = Timestamp::now();

        let settings = TinyflakeSettings::builder()
            .node_id(0)
            .start_epoch(epoch)
            .build();

        let tinyflake = Tinyflake::new(settings).unwrap();

        let first = tinyflake.generate();
        let second = tinyflake.generate();

        assert!(matches!(first, ShortCode::Generated(_)));
        assert!(matches!(second, ShortCode::Generated(_)));
        assert_ne!(first.as_str(), second.as_str());
    }
}
