use crate::base58::ShortCodeBase58;
use modular_bitfield::prelude::*;

#[bitfield]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SlimId {
    /// 30 bits for timestamp (seconds since a custom epoch)
    pub timestamp: B30,
    /// 8 bits for sequence number (resets every second)
    pub sequence: B8,
    /// 2 bits for node ID (allows up to 4 nodes)
    pub node_id: B2,
}

impl From<SlimId> for ShortCodeBase58 {
    fn from(val: SlimId) -> Self {
        let bytes = val.into_bytes();
        ShortCodeBase58::new(bytes)
    }
}
