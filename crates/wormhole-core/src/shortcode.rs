use crate::base58::ShortCodeBase58;
use crate::error::ShortenerError;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// A validated short code identifier for a shortened URL.
///
/// Short codes must be 3-32 characters long and contain only
/// alphanumeric characters, hyphens, or underscores.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShortCode {
    /// A system-generated short code (e.g. from an ID generator).
    Generated(ShortCodeBase58),
    /// A user-provided custom short code.
    Custom(String),
}

const MIN_LENGTH: usize = 3;
const MAX_LENGTH: usize = 32;

impl ShortCode {
    /// Creates a `ShortCode` from a value that can be converted into [`ShortCodeBase58`].
    ///
    /// This accepts a [`ShortCodeBase58`] directly, or a [`SlimId`] which will be
    /// automatically encoded as base58.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // From a SlimId
    /// let slim_id = SlimId::new().with_timestamp(12345).with_sequence(1);
    /// let code = ShortCode::generated(slim_id);
    ///
    /// // From ShortCodeBase58
    /// let base58 = ShortCodeBase58::new(bytes);
    /// let code = ShortCode::generated(base58);
    /// ```
    pub fn generated(code: impl Into<ShortCodeBase58>) -> Self {
        Self::Generated(code.into())
    }

    /// Creates a new `ShortCode` after validating the input.
    ///
    /// Valid codes are 3-32 characters and contain only `[a-zA-Z0-9_-]`.
    pub fn new(code: impl Into<String>) -> std::result::Result<Self, ShortenerError> {
        let code = code.into();
        Self::validate(&code)?;
        Ok(Self::Custom(code))
    }

    /// Creates a `ShortCode` without validation.
    ///
    /// Use this only for codes produced by trusted internal sources
    /// (e.g. ID generators that are guaranteed to produce valid output).
    pub fn new_unchecked(code: impl Into<String>) -> Self {
        Self::Custom(code.into())
    }

    /// Generates the full shortened URL based on the provided base URL.
    pub fn to_url(&self, base_url: &str) -> String {
        format!("{}/{}", base_url.trim_end_matches('/'), self)
    }

    /// Returns the short code as a string slice.
    pub fn as_str(&self) -> &str {
        match self {
            ShortCode::Generated(tiny) => tiny.as_str(),
            ShortCode::Custom(s) => s.as_str(),
        }
    }

    fn validate(code: &str) -> std::result::Result<(), ShortenerError> {
        if code.len() < MIN_LENGTH || code.len() > MAX_LENGTH {
            return Err(ShortenerError::InvalidShortCode(format!(
                "length must be between {} and {}, got {}",
                MIN_LENGTH,
                MAX_LENGTH,
                code.len()
            )));
        }

        if !code
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(ShortenerError::InvalidShortCode(format!(
                "must contain only alphanumeric characters, hyphens, or underscores: '{}'",
                code
            )));
        }

        Ok(())
    }
}

impl Display for ShortCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShortCode::Generated(tiny) => write!(f, "{}", tiny),
            ShortCode::Custom(s) => f.write_str(s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::slim_id::SlimId;

    #[test]
    fn valid_codes() {
        assert!(ShortCode::new("abc").is_ok());
        assert!(ShortCode::new("Abc-123_xyz").is_ok());
        assert!(ShortCode::new("a".repeat(32)).is_ok());
    }

    #[test]
    fn too_short() {
        assert!(ShortCode::new("ab").is_err());
        assert!(ShortCode::new("").is_err());
    }

    #[test]
    fn too_long() {
        assert!(ShortCode::new("a".repeat(33)).is_err());
    }

    #[test]
    fn invalid_characters() {
        assert!(ShortCode::new("abc def").is_err());
        assert!(ShortCode::new("abc/def").is_err());
        assert!(ShortCode::new("abc!def").is_err());
    }

    #[test]
    fn display_custom() {
        let code = ShortCode::new("my-code").unwrap();
        assert_eq!(code.to_string(), "my-code");
    }

    #[test]
    fn display_generated() {
        let slim_id = SlimId::new()
            .with_timestamp(12345)
            .with_sequence(1)
            .with_node_id(0);
        let code = ShortCode::generated(slim_id);
        // ShortCodeBase58 uses base58 encoding
        assert!(!code.to_string().is_empty());
    }

    #[test]
    fn to_url_custom() {
        let code = ShortCode::new("abc123").unwrap();
        assert_eq!(code.to_url("https://worm.hole"), "https://worm.hole/abc123");
        assert_eq!(
            code.to_url("https://worm.hole/"),
            "https://worm.hole/abc123"
        );
    }
}
