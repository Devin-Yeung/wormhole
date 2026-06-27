use crate::{
    clock::{Clock, SystemClock},
    error::Error,
    TinyId,
};
use jiff::Timestamp;
use std::sync::Mutex;
use typed_builder::TypedBuilder;

const MAX_TIMESTAMP_SECONDS: u64 = (1_u64 << 30) - 1;
const MAX_NODE_ID: u8 = 0b11;
const MAX_SEQUENCE: u8 = u8::MAX;

/// Configures a Tinyflake generator instance.
#[derive(Debug, Clone, Copy, TypedBuilder)]
pub struct TinyflakeSettings {
    /// A unique node index in the range `[0, 3]`.
    #[builder]
    pub node_id: u8,
    /// Custom epoch used as the zero point for the 30-bit timestamp field.
    ///
    /// Tinyflake math runs at whole-second precision (`Timestamp::as_second`).
    /// Sub-second detail is intentionally not modeled in the 30-bit timestamp.
    #[builder]
    pub start_epoch: Timestamp,
}

#[derive(Debug, Default)]
struct GeneratorState {
    last_elapsed_timestamp: Option<Timestamp>,
    sequence: u8,
}

/// Tinyflake ID generator with Sonyflake-style wait-on-overflow semantics.
pub struct Tinyflake<C: Clock> {
    start_time: Timestamp,
    node_id: u8,
    clock: C,
    state: Mutex<GeneratorState>,
}

impl Tinyflake<SystemClock> {
    /// Creates a generator backed by the real system clock.
    pub fn new(settings: TinyflakeSettings) -> Result<Self, Error> {
        Self::with_clock(settings, SystemClock)
    }
}

impl<C: Clock> Tinyflake<C> {
    fn with_clock(settings: TinyflakeSettings, clock: C) -> Result<Self, Error> {
        if settings.node_id > MAX_NODE_ID {
            return Err(Error::InvalidNodeId {
                node_id: settings.node_id,
                max_node_id: MAX_NODE_ID,
            });
        }

        let now = clock.now();
        if settings.start_epoch > now {
            return Err(Error::EpochAhead {
                epoch: settings.start_epoch,
                now,
            });
        }

        Ok(Self {
            start_time: settings.start_epoch,
            node_id: settings.node_id,
            clock,
            state: Mutex::new(GeneratorState::default()),
        })
    }

    /// Generates the next unique TinyId.
    ///
    /// Correctness strategy (matching Sonyflake behavior):
    /// - if the per-second sequence is exhausted, wait for the next second
    /// - if clock moves backward, wait until clock catches up
    pub fn next_id(&self) -> Result<TinyId, Error> {
        let mut state = self.state.lock().map_err(|_| Error::StatePoisoned)?;

        let mut now = self.clock.now();

        match state.last_elapsed_timestamp {
            None => {
                // First call: sequence starts at 0 (already the default).
                state.sequence = 0;
            }
            Some(last) => {
                if now < last {
                    // Clock moved backward — block until we've caught up to the
                    // last timestamp used. Without this, two calls could produce
                    // the same (timestamp, sequence, node_id) triple.
                    self.clock.wait_until(last);
                    now = self.clock.now();
                }

                if now.as_second() == last.as_second() {
                    if state.sequence < MAX_SEQUENCE {
                        state.sequence += 1;
                    } else {
                        // Per-second sequence exhausted: wait for the next
                        // second boundary, then reset so we start fresh.
                        let next_second = Timestamp::from_second(last.as_second() + 1)
                            .expect("next second is a valid timestamp");
                        self.clock.wait_until(next_second);
                        now = self.clock.now();
                        state.sequence = 0;
                    }
                } else {
                    // Entered a new second: the sequence counter resets.
                    state.sequence = 0;
                }
            }
        }

        // Seconds elapsed since the custom epoch, used as the timestamp field.
        let elapsed = now.as_second() - self.start_time.as_second();
        if elapsed as u64 > MAX_TIMESTAMP_SECONDS {
            return Err(Error::OverTimeLimit);
        }

        let id = TinyId::new()
            .with_timestamp(elapsed as u32)
            .with_sequence(state.sequence)
            .with_node_id(self.node_id);

        state.last_elapsed_timestamp = Some(now);

        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::test_clock::TestClock;

    fn make_generator(node_id: u8, clock_second: i64) -> Tinyflake<TestClock> {
        let epoch = Timestamp::from_second(0).unwrap();
        let settings = TinyflakeSettings::builder()
            .node_id(node_id)
            .start_epoch(epoch)
            .build();
        let clock = TestClock::new(Timestamp::from_second(clock_second).unwrap());
        Tinyflake::with_clock(settings, clock).unwrap()
    }

    #[test]
    fn first_id_has_sequence_zero() {
        let gen = make_generator(0, 100);
        let id = gen.next_id().unwrap();
        assert_eq!(id.sequence(), 0);
    }

    #[test]
    fn same_second_increments_sequence() {
        let gen = make_generator(0, 100);
        let id0 = gen.next_id().unwrap();
        let id1 = gen.next_id().unwrap();
        let id2 = gen.next_id().unwrap();
        assert_eq!(id0.sequence(), 0);
        assert_eq!(id1.sequence(), 1);
        assert_eq!(id2.sequence(), 2);
    }

    #[test]
    fn sequence_overflow_advances_clock() {
        let gen = make_generator(0, 100);
        // Exhaust all 256 IDs allocated to second 100.
        for _ in 0..=255 {
            gen.next_id().unwrap();
        }
        // The 257th call must wait for second 101; sequence resets to 0.
        let id = gen.next_id().unwrap();
        assert_eq!(id.sequence(), 0);
        assert_eq!(id.timestamp(), 101); // elapsed = 101s - epoch(0s)
    }

    #[test]
    fn node_id_is_embedded() {
        let gen = make_generator(3, 100);
        let id = gen.next_id().unwrap();
        assert_eq!(id.node_id(), 3);
    }

    #[test]
    fn timestamp_field_reflects_elapsed_seconds() {
        let gen = make_generator(0, 500);
        let id = gen.next_id().unwrap();
        // elapsed = 500s - epoch(0s)
        assert_eq!(id.timestamp(), 500);
    }

    #[test]
    fn overtime_limit_returns_error() {
        let epoch = Timestamp::from_second(0).unwrap();
        let settings = TinyflakeSettings::builder()
            .node_id(0)
            .start_epoch(epoch)
            .build();
        // Place the clock one second past the 30-bit timestamp limit.
        let over_limit = MAX_TIMESTAMP_SECONDS as i64 + 1;
        let clock = TestClock::new(Timestamp::from_second(over_limit).unwrap());
        let gen = Tinyflake::with_clock(settings, clock).unwrap();
        assert_eq!(gen.next_id(), Err(Error::OverTimeLimit));
    }
}
