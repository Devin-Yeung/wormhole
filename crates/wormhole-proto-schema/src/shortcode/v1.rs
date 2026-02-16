use std::convert::TryInto;
use wormhole_core::ShortenerError;

tonic::include_proto!("shortcode.v1");

impl TryInto<wormhole_core::ShortCode> for &ShortCode {
    type Error = ShortenerError;

    fn try_into(self) -> Result<wormhole_core::ShortCode, Self::Error> {
        let kind = ShortCodeKind::try_from(self.kind).map_err(|e| {
            ShortenerError::InvalidShortCode(format!("invalid short code kind: {}", e))
        })?;

        match kind {
            ShortCodeKind::Generated => {
                // We decode then re-encode to preserve the generated variant while
                // ensuring the wire value is valid base58.
                let decoded = bs58::decode(self.code.as_str()).into_vec().map_err(|e| {
                    ShortenerError::InvalidShortCode(format!("invalid base58 short code: {}", e))
                })?;
                Ok(wormhole_core::ShortCode::generated(
                    wormhole_core::base58::ShortCodeBase58::new(decoded),
                ))
            }
            ShortCodeKind::Custom => wormhole_core::ShortCode::new(self.code.as_str()),
        }
    }
}

impl TryInto<wormhole_core::ShortCode> for ShortCode {
    type Error = ShortenerError;

    fn try_into(self) -> Result<wormhole_core::ShortCode, Self::Error> {
        (&self).try_into()
    }
}

#[cfg(test)]
mod tests {
    use crate::v1::{ShortCode, ShortCodeKind};

    #[test]
    fn test_short_code_try_into() {
        let shortcode = ShortCode {
            code: "3mJr7A".to_string(),
            kind: ShortCodeKind::Generated as i32,
        };

        let result: wormhole_core::ShortCode =
            shortcode.try_into().expect("Failed to convert ShortCode");

        assert!(matches!(result, wormhole_core::ShortCode::Generated(_)));
    }

    #[test]
    fn invalid_base58() {
        let shortcode = ShortCode {
            code: "invalid_base58".to_string(),
            kind: ShortCodeKind::Generated as i32,
        };

        let result: Result<wormhole_core::ShortCode, _> = shortcode.try_into();
        assert!(result.is_err());
    }
}
