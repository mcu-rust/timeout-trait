use super::{
    TickDuration,
    fugit::{KilohertzU32, RateExtU32},
    prelude::*,
};
use std::time::{Duration, Instant};

/// [`TimeoutNs`] implementation.
#[derive(Default)]
pub struct StdTimeoutNs {}

impl StdTimeoutNs {
    pub const fn new() -> Self {
        Self {}
    }
}

impl TimeoutNs for StdTimeoutNs {
    type TimeoutState = StdTimeoutState;

    fn start_ns(&self, timeout: u32) -> Self::TimeoutState {
        StdTimeoutState {
            timeout: Duration::from_nanos(timeout.into()),
            start_time: Instant::now(),
        }
    }

    fn start_us(&self, timeout: u32) -> Self::TimeoutState {
        StdTimeoutState {
            timeout: Duration::from_micros(timeout.into()),
            start_time: Instant::now(),
        }
    }

    fn start_ms(&self, timeout: u32) -> Self::TimeoutState {
        StdTimeoutState {
            timeout: Duration::from_millis(timeout.into()),
            start_time: Instant::now(),
        }
    }
}

/// [`TimeoutState`] implementation for.
pub struct StdTimeoutState {
    timeout: Duration,
    start_time: Instant,
}

impl TimeoutState for StdTimeoutState {
    #[inline]
    fn timeout(&mut self) -> bool {
        if self.start_time.elapsed() >= self.timeout {
            self.start_time += self.timeout;
            true
        } else {
            false
        }
    }

    #[inline(always)]
    fn restart(&mut self) {
        self.start_time = Instant::now();
    }
}

#[derive(Clone)]
pub struct StdTickInstant(Instant);

impl TickInstant for StdTickInstant {
    #[inline]
    fn frequency() -> KilohertzU32 {
        1.MHz()
    }

    #[inline]
    fn now() -> Self {
        StdTickInstant(Instant::now())
    }

    #[inline]
    fn elapsed(&mut self) -> TickDuration<Self> {
        let ticks = Instant::now().duration_since(self.0).as_micros() as u64;
        TickDuration::from_ticks(ticks)
    }

    #[inline]
    fn add(&self, dur: &TickDuration<Self>) -> Self {
        Self(
            self.0
                .checked_add(Duration::from_micros(dur.ticks()))
                .unwrap(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TickTimeoutNs;
    use std::{thread::sleep, time::Duration};

    struct UseTimeout<T: TimeoutState> {
        interval: T,
    }

    fn test_timeout<T: TimeoutNs>(timeout: T) {
        let mut t = timeout.start_ms(500);
        assert!(!t.timeout());
        sleep(Duration::from_millis(260));
        assert!(!t.timeout());
        sleep(Duration::from_millis(260));
        assert!(t.timeout());
        assert!(!t.timeout());

        t.restart();
        assert!(!t.timeout());
        sleep(Duration::from_millis(260));
        assert!(!t.timeout());
        sleep(Duration::from_millis(260));
        assert!(t.timeout());
        assert!(!t.timeout());

        assert!(timeout.ns_with(100, || {
            sleep(Duration::from_nanos(1));
            true
        }));

        let mut u = UseTimeout {
            interval: timeout.start_ms(1),
        };
        u.interval.timeout();
    }

    #[test]
    fn std_timeout() {
        test_timeout(StdTimeoutNs {});
    }

    #[test]
    fn tick_timeout() {
        test_timeout(TickTimeoutNs::<StdTickInstant>::new());
    }

    #[test]
    fn tick_instant() {
        let mut now = StdTickInstant::now();
        sleep(Duration::from_millis(200));
        assert!(now.elapsed().ticks() - 200_000 < 1000);
    }
}
