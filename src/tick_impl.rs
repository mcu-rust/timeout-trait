use super::{TickDuration, prelude::*};
use core::marker::PhantomData;

#[derive(Default)]
pub struct TickTimeoutNs<T> {
    _t: PhantomData<T>,
}

impl<T> TickTimeoutNs<T>
where
    T: TickInstant,
{
    pub const fn new() -> Self {
        Self { _t: PhantomData }
    }
}

impl<T> TimeoutNs for TickTimeoutNs<T>
where
    T: TickInstant,
{
    type TimeoutState = TickTimeoutState<T>;

    #[inline]
    fn start_ns(&self, timeout: u32) -> Self::TimeoutState {
        TickTimeoutState::<T>::new_nanos(timeout)
    }

    #[inline]
    fn start_us(&self, timeout: u32) -> Self::TimeoutState {
        TickTimeoutState::<T>::new_micros(timeout)
    }

    #[inline]
    fn start_ms(&self, timeout: u32) -> Self::TimeoutState {
        TickTimeoutState::<T>::new_millis(timeout)
    }
}

pub struct TickTimeoutState<T: TickInstant> {
    time: T,
    timeout: TickDuration<T>,
}

impl<T> TickTimeoutState<T>
where
    T: TickInstant,
{
    pub fn new_nanos(timeout: u32) -> Self {
        Self::new(TickDuration::from_nanos(timeout))
    }

    pub fn new_micros(timeout: u32) -> Self {
        Self::new(TickDuration::from_micros(timeout))
    }

    pub fn new_millis(timeout: u32) -> Self {
        Self::new(TickDuration::from_millis(timeout))
    }

    fn new(timeout: TickDuration<T>) -> Self {
        Self {
            time: T::now(),
            timeout,
        }
    }
}

impl<T> TimeoutState for TickTimeoutState<T>
where
    T: TickInstant,
{
    /// Can be reused without calling `restart()`.
    #[inline]
    fn timeout(&mut self) -> bool {
        if self.time.timeout(&self.timeout) {
            self.time = self.time.add(&self.timeout);
            return true;
        }
        false
    }

    #[inline(always)]
    fn restart(&mut self) {
        self.time = T::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::sync::atomic::{AtomicU64, Ordering};
    use fugit::{ExtU32, KilohertzU32, NanosDurationU32, RateExtU32};

    static TICK_SOURCE: AtomicU64 = AtomicU64::new(0);

    #[derive(Clone, Copy)]
    pub struct MockInstant {
        tick: u64,
    }

    impl MockInstant {
        fn add_time(t: NanosDurationU32) {
            let tick = t.to_nanos() as u64;
            TICK_SOURCE.fetch_add(tick, Ordering::Relaxed);
        }
    }

    impl TickInstant for MockInstant {
        fn frequency() -> KilohertzU32 {
            1000.MHz()
        }

        fn now() -> Self {
            Self {
                tick: TICK_SOURCE.load(Ordering::Relaxed),
            }
        }

        fn elapsed(&mut self) -> TickDuration<Self> {
            TickDuration::from_ticks(Self::now().tick.wrapping_sub(self.tick))
        }

        fn add(&self, dur: &TickDuration<Self>) -> Self {
            Self {
                tick: self.tick + dur.ticks(),
            }
        }
    }

    #[test]
    fn tick_timeout() {
        let mut now = MockInstant::now();
        assert_eq!(now.elapsed().ticks(), 0);
        MockInstant::add_time(10.nanos());
        assert_eq!(now.elapsed().ticks(), 10);
        MockInstant::add_time(1.millis());
        assert_eq!(now.elapsed().ticks(), 1_000_010);
        MockInstant::add_time(10.micros());
        let mut now2 = MockInstant::now();
        assert_eq!(now.elapsed().ticks(), 1_010_010);
        MockInstant::add_time(10.micros());
        assert_eq!(now2.elapsed().ticks(), 10_000);

        let mut t = TickTimeoutNs::<MockInstant>::new().start_ns(100);
        assert!(!t.timeout());
        MockInstant::add_time(10.nanos());
        assert!(!t.timeout());
        MockInstant::add_time(90.nanos());
        assert!(t.timeout());
        assert!(!t.timeout());
        MockInstant::add_time(100.nanos());
        assert!(t.timeout());

        MockInstant::add_time(90.nanos());
        assert!(!t.timeout());
        t.restart();
        MockInstant::add_time(90.nanos());
        assert!(!t.timeout());
        MockInstant::add_time(10.nanos());
        assert!(t.timeout());

        let mut t = TickTimeoutNs::<MockInstant>::new().start_us(100);
        assert!(!t.timeout());
        MockInstant::add_time(10.micros());
        assert!(!t.timeout());
        MockInstant::add_time(90.micros());
        assert!(t.timeout());
        assert!(!t.timeout());
        MockInstant::add_time(100.micros());
        assert!(t.timeout());

        MockInstant::add_time(90.micros());
        assert!(!t.timeout());
        t.restart();
        MockInstant::add_time(90.micros());
        assert!(!t.timeout());
        MockInstant::add_time(10.micros());
        assert!(t.timeout());

        let mut t = TickTimeoutNs::<MockInstant>::new().start_ms(100);
        assert!(!t.timeout());
        MockInstant::add_time(10.millis());
        assert!(!t.timeout());
        MockInstant::add_time(90.millis());
        assert!(t.timeout());
        assert!(!t.timeout());
        MockInstant::add_time(100.millis());
        assert!(t.timeout());

        MockInstant::add_time(90.millis());
        assert!(!t.timeout());
        t.restart();
        MockInstant::add_time(90.millis());
        assert!(!t.timeout());
        MockInstant::add_time(10.millis());
        assert!(t.timeout());

        let mut count = 0;
        let t = TickTimeoutNs::<MockInstant>::new();
        assert!(t.ns_with(100, || {
            MockInstant::add_time(10.nanos());
            count += 1;
            true
        }));
        assert_eq!(count, 10);

        let t = TickTimeoutState::<MockInstant>::new_micros(40_000_000);
        assert_eq!(t.timeout.ticks(), 40_000_000_000);

        let mut t = TickTimeoutState::<MockInstant>::new_millis(40_000);
        assert_eq!(t.timeout.ticks(), 40_000_000_000);

        assert!(!t.timeout());

        for _ in 0..40 {
            MockInstant::add_time(999.millis());
            assert!(!t.timeout());
        }
        MockInstant::add_time(100.millis());
        assert!(t.timeout());
        assert!(!t.timeout());

        for _ in 0..39 {
            MockInstant::add_time(999.millis());
            assert!(!t.timeout());
        }
        t.restart();
        for _ in 0..40 {
            MockInstant::add_time(999.millis());
            assert!(!t.timeout());
        }
        MockInstant::add_time(100.millis());
        assert!(t.timeout());
    }
}

#[cfg(test)]
mod tests_fugit {
    use core::ops::Div;
    use fugit::{
        ExtU32, ExtU32Ceil, KilohertzU32, MicrosDurationU32, MillisDurationU32, NanosDurationU64,
        RateExtU32,
    };

    #[test]
    fn duration_tick() {
        assert_eq!(1 / 1000, 0);
        assert_eq!(1_u32.div(1000), 0);
        assert_eq!(1_u32.div_ceil(1000), 1);

        let dur: MicrosDurationU32 = 1.micros();
        assert_eq!(dur.ticks(), 1);

        let dur: MicrosDurationU32 = 1.millis();
        assert_eq!(dur.ticks(), 1000);
        assert_eq!(dur.to_millis(), 1);

        let dur: MillisDurationU32 = 1.millis();
        assert_eq!(dur.ticks(), 1);
        assert_eq!(dur.to_micros(), 1000);

        let dur: MillisDurationU32 = 1.micros();
        assert_eq!(dur.ticks(), 0);

        let dur: MillisDurationU32 = 1.micros_at_least();
        assert_eq!(dur.ticks(), 1);

        let dur: MicrosDurationU32 = 1.micros();
        assert_eq!(dur.to_millis(), 0);
        let dur: MillisDurationU32 = dur.ticks().micros_at_least();
        assert_eq!(dur.ticks(), 1);

        let a = MicrosDurationU32::micros(100);
        let b = MicrosDurationU32::micros(99);
        assert!(b < a);

        let a = NanosDurationU64::micros(100);
        let b = NanosDurationU64::micros(101);
        assert!(b > a);
    }

    #[test]
    fn rate_tick() {
        let r: KilohertzU32 = 1.MHz();
        assert_eq!(r.raw(), 1_000);
        let f: u32 = r.to_Hz();
        assert_eq!(f, 1_000_000);
    }
}
