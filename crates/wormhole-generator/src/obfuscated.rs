use crate::Generator;
use typed_builder::TypedBuilder;
use wormhole_core::base58::ShortCodeBase58;
use wormhole_core::ShortCode;
use wormhole_tinyflake::{Clock, SystemClock, TinyId, Tinyflake, TinyflakeSettings};

const LOWER_40_BITS_MASK: u64 = (1_u64 << 40) - 1;

#[derive(Debug, TypedBuilder)]
/// An Obfuscator that specially design for obfuscating TinyID.
/// It uses a simple multiplicative and XOR-based obfuscation method.
pub struct Obfuscator {
    #[builder(default = 3)]
    prime: u64,
    #[builder(default = 0xDEAD_BEEF_CAFE_BABE)]
    mask: u64,
}

impl Obfuscator {
    pub fn prime(&self) -> u64 {
        self.prime
    }

    pub fn mask(&self) -> u64 {
        self.mask
    }

    pub fn obfuscate(&self, id: TinyId) -> ObfuscatedTinyID {
        let raw = id.into_bytes();
        let source = u64::from_be_bytes([0, 0, 0, raw[0], raw[1], raw[2], raw[3], raw[4]]);

        let obfuscated = (source.wrapping_mul(self.prime) ^ self.mask) & LOWER_40_BITS_MASK;
        let obfuscated_bytes = obfuscated.to_be_bytes();

        ObfuscatedTinyID {
            inner: [
                obfuscated_bytes[3],
                obfuscated_bytes[4],
                obfuscated_bytes[5],
                obfuscated_bytes[6],
                obfuscated_bytes[7],
            ],
        }
    }
}

pub struct ObfuscatedTinyID {
    inner: [u8; 5],
}

impl Into<ShortCodeBase58> for ObfuscatedTinyID {
    fn into(self) -> ShortCodeBase58 {
        ShortCodeBase58::new(self.inner)
    }
}

impl Into<ShortCode> for ObfuscatedTinyID {
    fn into(self) -> ShortCode {
        ShortCode::Generated(self.into())
    }
}

pub struct ObfuscatedTinyFlake<C: Clock> {
    inner: Tinyflake<C>,
    obfuscator: Obfuscator,
}

impl ObfuscatedTinyFlake<SystemClock> {
    pub fn new(settings: TinyflakeSettings, obfuscator: Obfuscator) -> Self {
        Self {
            inner: Tinyflake::new(settings).unwrap(),
            obfuscator,
        }
    }
}

impl<C: Clock> ObfuscatedTinyFlake<C> {
    pub fn next_obfuscated_id(&self) -> ObfuscatedTinyID {
        let id = self.inner.next_id().unwrap(); // TODO: safe unwrap?
        self.obfuscator.obfuscate(id)
    }
}

impl<C: Clock + 'static> Generator for ObfuscatedTinyFlake<C> {
    type Output = ObfuscatedTinyID;

    fn generate(&self) -> Self::Output {
        self.next_obfuscated_id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::Timestamp;
    use wormhole_tinyflake::{Tinyflake, TinyflakeSettings};

    fn pack_u40_be(bytes: [u8; 5]) -> u64 {
        u64::from_be_bytes([0, 0, 0, bytes[0], bytes[1], bytes[2], bytes[3], bytes[4]])
    }

    fn unpack_u40_be(value: u64) -> [u8; 5] {
        let raw = value.to_be_bytes();
        [raw[3], raw[4], raw[5], raw[6], raw[7]]
    }

    #[test]
    fn obfuscate_applies_multiplication_xor_in_u40_space() {
        let id = TinyId::new()
            .with_timestamp(0x3FFF_FFFF)
            .with_sequence(0xA5)
            .with_node_id(0b11);

        let obfuscator = Obfuscator::builder().build();

        let obfuscated = obfuscator.obfuscate(id);

        let source = pack_u40_be(id.into_bytes());
        let expected =
            (source.wrapping_mul(obfuscator.prime()) ^ obfuscator.mask()) & LOWER_40_BITS_MASK;
        assert_eq!(obfuscated.inner, unpack_u40_be(expected));
    }

    #[test]
    fn obfuscated_tiny_id_converts_into_base58() {
        let obfuscated = ObfuscatedTinyID {
            inner: [0x10, 0x20, 0x30, 0x40, 0x50],
        };

        let code: ShortCodeBase58 = obfuscated.into();

        assert_eq!(
            code.as_str(),
            ShortCodeBase58::new([0x10, 0x20, 0x30, 0x40, 0x50]).as_str()
        );
    }

    #[test]
    fn tinyid_to_shortcode_base58_via_obfuscator() {
        let start: Timestamp = "2026-01-01T00:00:00+08[Asia/Shanghai]".parse().unwrap();

        let settings = TinyflakeSettings::builder()
            .node_id(0)
            .start_epoch(start)
            .build();

        let tinyflake = Tinyflake::new(settings).unwrap();

        let obfuscator = Obfuscator::builder().build();

        let first: ShortCodeBase58 = obfuscator.obfuscate(tinyflake.next_id().unwrap()).into();
        let second: ShortCodeBase58 = obfuscator.obfuscate(tinyflake.next_id().unwrap()).into();

        assert_ne!(first.as_str(), second.as_str());
    }
}
