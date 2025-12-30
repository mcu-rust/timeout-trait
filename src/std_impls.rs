use crate::{
    fugit::{KilohertzU32, RateExtU32},
    *,
};
use std::time::{Duration, Instant};

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
    fn move_forward(&mut self, dur: &TickDuration<Self>) {
        self.0 = self
            .0
            .checked_add(Duration::from_micros(dur.ticks()))
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use crate::TickTimeout;

    use super::*;
    use std::{thread::sleep, time::Duration};

    struct UseTimeout<T: TickInstant> {
        interval: TickTimeout<T>,
    }

    fn test_timeout<T: TickInstant>() {
        let mut t = TickTimeout::<T>::from_millis(500);
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

        let dur = TickDuration::<T>::from_nanos(100);
        assert!(T::now().timeout_with(&dur, || {
            sleep(Duration::from_nanos(1));
            true
        }));

        let mut u = UseTimeout {
            interval: TickTimeout::<T>::from_millis(1),
        };
        u.interval.timeout();
    }

    #[test]
    fn tick_timeout() {
        test_timeout::<StdTickInstant>();
    }

    #[test]
    fn tick_instant() {
        let mut now = StdTickInstant::now();
        sleep(Duration::from_millis(200));
        assert!(now.elapsed().ticks() - 200_000 < 1000);
    }
}
