use jiff::Timestamp;
use std::time::Duration;

pub trait Clock: Send + Sync {
    /// Returns the current time of the clock
    fn now(&self) -> Timestamp;
    /// Block and wait until the clock reaches the target time.
    fn wait_until(&self, target: Timestamp);
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        Timestamp::now()
    }

    fn wait_until(&self, target: Timestamp) {
        // Poll in a loop to handle spurious wakeups. The loop condition is
        // re-evaluated after each sleep so we don't over-sleep past the target.
        loop {
            let now = Timestamp::now();
            if now >= target {
                return;
            }
            // Sleep the remaining whole seconds (converted to ms). A minimum of
            // 1 ms prevents busy-waiting when the gap is sub-millisecond.
            let remaining_ms = ((target.as_second() - now.as_second()) * 1_000).max(1) as u64;
            std::thread::sleep(Duration::from_millis(remaining_ms));
        }
    }
}

#[cfg(test)]
pub(crate) mod test_clock {
    use crate::clock::Clock;
    use jiff::Timestamp;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    pub(crate) struct TestClock {
        inner: Arc<Mutex<TestClockState>>,
    }

    struct TestClockState {
        now: Timestamp,
    }

    impl TestClock {
        pub(crate) fn new(now: Timestamp) -> Self {
            Self {
                inner: Arc::new(Mutex::new(TestClockState { now })),
            }
        }
    }

    impl Clock for TestClock {
        fn now(&self) -> Timestamp {
            self.inner
                .lock()
                .expect("test clock lock should not be poisoned")
                .now
        }

        fn wait_until(&self, target: Timestamp) {
            let mut state = self
                .inner
                .lock()
                .expect("test clock lock should not be poisoned");
            // just advance the clock to the target time;
            // we don't need to actually block since this is only used in tests
            if target > state.now {
                state.now = target;
            }
        }
    }

    #[test]
    fn test_clock_works() {
        // test that the clock starts at the given time
        let base = Timestamp::from_second(0).unwrap();
        let clock = TestClock::new(base);
        assert_eq!(clock.now(), base);

        // the clock should advance to the target time after wait_until
        let target = Timestamp::from_second(1000).unwrap();
        clock.wait_until(target);
        assert_eq!(clock.now(), target);
    }
}
