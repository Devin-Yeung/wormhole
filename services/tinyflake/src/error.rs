use jiff::Timestamp;
use thiserror::Error;

/// Errors returned by Tinyflake initialization and ID generation.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum Error {
    #[error("invalid node id {node_id}; expected 0..={max_node_id}")]
    InvalidNodeId { node_id: u8, max_node_id: u8 },
    #[error("epoch is ahead of current clock time: epoch={epoch}, now={now}")]
    EpochAhead { epoch: Timestamp, now: Timestamp },
    #[error("overtime limit")]
    OverTimeLimit,
    #[error("generator state lock is poisoned")]
    StatePoisoned,
}
