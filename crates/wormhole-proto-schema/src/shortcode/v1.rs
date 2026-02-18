use std::convert::TryInto;
use thiserror::Error;
use wormhole_core as core;
use wormhole_core::base58::ShortCodeBase58;

tonic::include_proto!("shortcode.v1");

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("invalid short code kind: {0}")]
    InvalidKind(i32),
    #[error("short code is malformed: {0}")]
    MalformedCode(String),
}

impl TryInto<core::ShortCode> for &ShortCode {
    type Error = ConversionError;

    fn try_into(self) -> Result<core::ShortCode, Self::Error> {
        let kind = ShortCodeKind::try_from(self.kind)
            .map_err(|_| ConversionError::InvalidKind(self.kind))?;

        match kind {
            ShortCodeKind::Generated => {
                // We decode then re-encode to preserve the generated variant while
                // ensuring the wire value is valid base58.
                let decoded = bs58::decode(self.code.as_str()).into_vec().map_err(|e| {
                    ConversionError::MalformedCode(format!(
                        "failed to decode base58 short code: {e}"
                    ))
                })?;
                Ok(core::ShortCode::generated(ShortCodeBase58::new(decoded)))
            }
            ShortCodeKind::Custom => core::ShortCode::new(self.code.as_str())
                .map_err(|_| ConversionError::MalformedCode(self.code.clone())),
        }
    }
}

impl TryInto<core::ShortCode> for ShortCode {
    type Error = ConversionError;

    fn try_into(self) -> Result<core::ShortCode, Self::Error> {
        (&self).try_into()
    }
}

#[cfg(test)]
mod tests {
    use crate::v1::{ShortCode, ShortCodeKind};
    use wormhole_core as core;

    #[test]
    fn test_short_code_try_into() {
        let shortcode = ShortCode {
            code: "3mJr7A".to_string(),
            kind: ShortCodeKind::Generated as i32,
        };

        let result: core::ShortCode = shortcode.try_into().expect("Failed to convert ShortCode");

        assert!(matches!(result, core::ShortCode::Generated(_)));
    }

    #[test]
    fn invalid_base58() {
        let shortcode = ShortCode {
            code: "invalid_base58".to_string(),
            kind: ShortCodeKind::Generated as i32,
        };

        let result: Result<core::ShortCode, _> = shortcode.try_into();
        assert!(result.is_err());
    }
}
