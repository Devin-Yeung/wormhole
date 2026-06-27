use smol_str::SmolStr;
use std::fmt::Display;
use wormhole_tinyflake::TinyId;

/// A short code encoded as base58 string.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ShortCodeBase58(SmolStr);

impl ShortCodeBase58 {
    /// Creates a new `ShortCodeBase58` by encoding the given bytes as base58.
    ///
    /// # Type Parameters
    ///
    /// * `T` - A type that can be referenced as a byte slice (e.g., `[u8]`, `Vec<u8>`,
    ///   or the 8-byte array returned by [`TinyId::into_bytes`][wormhole_tinyflake::TinyId]).
    pub fn new<T: AsRef<[u8]>>(bytes: T) -> Self {
        let encoded = bs58::encode(bytes).into_string();
        Self(SmolStr::new(encoded))
    }

    /// Returns the short code as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for ShortCodeBase58 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ShortCodeBase58").field(&self.0).finish()
    }
}

impl Display for ShortCodeBase58 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<TinyId> for ShortCodeBase58 {
    fn from(val: TinyId) -> Self {
        let bytes = val.into_bytes();
        ShortCodeBase58::new(bytes)
    }
}
