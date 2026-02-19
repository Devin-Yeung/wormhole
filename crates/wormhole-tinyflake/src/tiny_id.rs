use modular_bitfield::prelude::*;

#[bitfield]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct TinyId {
    /// 30 bits for timestamp (seconds since a custom epoch)
    pub(crate) timestamp: B30,
    /// 8 bits for sequence number (resets every second)
    pub(crate) sequence: B8,
    /// 2 bits for node ID (allows up to 4 nodes)
    pub(crate) node_id: B2,
}
