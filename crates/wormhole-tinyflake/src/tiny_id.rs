use modular_bitfield::prelude::*;
use std::fmt;

#[bitfield]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct TinyId {
    /// 30 bits for timestamp (seconds since a custom epoch).
    pub timestamp: B30,
    /// 8 bits for sequence number (resets every second).
    pub sequence: B8,
    /// 2 bits for node ID (allows up to 4 nodes).
    pub node_id: B2,
}

impl fmt::Debug for TinyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TinyId")
            .field("timestamp", &self.timestamp())
            .field("sequence", &self.sequence())
            .field("node_id", &self.node_id())
            .finish()
    }
}
