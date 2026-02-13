use crate::error::{Error, Result, ShortenerError};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Minimum length for a short code.
const MIN_LENGTH: usize = 3;
/// Maximum length for a short code.
const MAX_LENGTH: usize = 32;

/// A validated short code identifier for a shortened URL.
///
/// Short codes must be 3-32 characters long and contain only
/// alphanumeric characters, hyphens, or underscores.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShortCode(String);

impl ShortCode {
    /// Creates a new `ShortCode` after validating the input.
    ///
    /// Valid codes are 3-32 characters and contain only `[a-zA-Z0-9_-]`.
    pub fn new(code: impl Into<String>) -> Result<Self> {
        let code = code.into();
        Self::validate(&code)?;
        Ok(Self(code))
    }

    /// Creates a `ShortCode` without validation.
    ///
    /// Use this only for codes produced by trusted internal sources
    /// (e.g. ID generators that are guaranteed to produce valid output).
    pub fn new_unchecked(code: impl Into<String>) -> Self {
        Self(code.into())
    }

    /// Returns the short code as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Generates the full shortened URL based on the provided base URL.
    pub fn to_url(&self, base_url: &str) -> String {
        format!("{}/{}", base_url.trim_end_matches('/'), self.0)
    }

    fn validate(code: &str) -> Result<()> {
        if code.len() < MIN_LENGTH || code.len() > MAX_LENGTH {
            return Err(Error::Shortener(ShortenerError::InvalidShortCode(format!(
                "length must be between {} and {}, got {}",
                MIN_LENGTH,
                MAX_LENGTH,
                code.len()
            ))));
        }

        if !code
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(Error::Shortener(ShortenerError::InvalidShortCode(format!(
                "must contain only alphanumeric characters, hyphens, or underscores: '{}'",
                code
            ))));
        }

        Ok(())
    }
}

impl fmt::Display for ShortCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for ShortCode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn display() {
        let code = ShortCode::new("my-code").unwrap();
        assert_eq!(code.to_string(), "my-code");
    }

    #[test]
    fn to_url() {
        let code = ShortCode::new("abc123").unwrap();
        assert_eq!(code.to_url("https://worm.hole"), "https://worm.hole/abc123");
        assert_eq!(
            code.to_url("https://worm.hole/"),
            "https://worm.hole/abc123"
        );
    }
}
