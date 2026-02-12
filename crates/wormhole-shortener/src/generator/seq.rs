use crate::generator::Generator;
use crate::shortcode::ShortCode;

/// A globally unique short code generator using sequential counters.
///
/// This generator produces sequential codes like "seq000", "seq001", etc.
/// It guarantees global uniqueness within a single instance (no database queries needed).
///
/// For distributed deployments, each node should use a unique prefix
/// (e.g., "node-a-000", "node-b-000") to ensure global uniqueness.
#[derive(Debug)]
pub struct UniqueGenerator {
    counter: std::sync::atomic::AtomicU64,
    prefix: String,
}

impl Clone for UniqueGenerator {
    fn clone(&self) -> Self {
        Self {
            counter: std::sync::atomic::AtomicU64::new(
                self.counter.load(std::sync::atomic::Ordering::SeqCst),
            ),
            prefix: self.prefix.clone(),
        }
    }
}

impl UniqueGenerator {
    /// Creates a new unique generator with a custom prefix.
    ///
    /// For distributed deployments, use unique prefixes per node
    /// to ensure global uniqueness (e.g., "node-a", "node-b").
    pub fn with_prefix(prefix: impl Into<String>) -> Self {
        Self {
            counter: std::sync::atomic::AtomicU64::new(0),
            prefix: prefix.into(),
        }
    }

    /// Creates a new unique generator starting from a specific counter value.
    ///
    /// Useful for resuming from a known state or distributing
    /// counter ranges across nodes (e.g., node 1 starts at 0, node 2 at 1_000_000).
    pub fn with_offset(prefix: impl Into<String>, offset: u64) -> Self {
        Self {
            counter: std::sync::atomic::AtomicU64::new(offset),
            prefix: prefix.into(),
        }
    }
}

impl Generator for UniqueGenerator {
    fn generate(&self) -> ShortCode {
        let count = self
            .counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        // Use base62 encoding for shorter, more readable codes
        let code_str = format!("{}{:06}", self.prefix, count);
        ShortCode::new_unchecked(code_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_generator_produces_sequential_codes() {
        let generator = UniqueGenerator::with_prefix("wh");

        let code1 = generator.generate();
        let code2 = generator.generate();
        let code3 = generator.generate();

        assert_eq!(code1.as_str(), "wh000000");
        assert_eq!(code2.as_str(), "wh000001");
        assert_eq!(code3.as_str(), "wh000002");
    }

    #[test]
    fn unique_generator_with_prefix() {
        let generator = UniqueGenerator::with_prefix("node-a");

        let code1 = generator.generate();
        let code2 = generator.generate();

        assert_eq!(code1.as_str(), "node-a000000");
        assert_eq!(code2.as_str(), "node-a000001");
    }

    #[test]
    fn unique_generator_with_offset() {
        let generator = UniqueGenerator::with_offset("wh", 1000);

        let code1 = generator.generate();
        let code2 = generator.generate();

        assert_eq!(code1.as_str(), "wh001000");
        assert_eq!(code2.as_str(), "wh001001");
    }

    #[test]
    fn generator_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<UniqueGenerator>();
    }

    #[test]
    fn clone_preserves_counter_state() {
        let generator = UniqueGenerator::with_prefix("wh");
        generator.generate();
        generator.generate();

        let cloned = generator.clone();

        // Original continues from 2
        assert_eq!(generator.generate().as_str(), "wh000002");

        // Clone also continues from 2 (same counter value)
        assert_eq!(cloned.generate().as_str(), "wh000002");
    }
}
