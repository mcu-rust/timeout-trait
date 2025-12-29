use super::prelude::*;
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
        TickTimeoutState::<T>::new_ns(timeout)
    }

    #[inline]
    fn start_us(&self, timeout: u32) -> Self::TimeoutState {
        TickTimeoutState::<T>::new_us(timeout)
    }

    #[inline]
    fn start_ms(&self, timeout: u32) -> Self::TimeoutState {
        TickTimeoutState::<T>::new_ms(timeout)
    }
}

pub struct TickTimeoutState<T: TickInstant> {
    tick: T,
    timeout_tick: u64,
    elapsed_tick: u64,
}

impl<T> TickTimeoutState<T>
where
    T: TickInstant,
{
    pub fn new_ns(timeout: u32) -> Self {
        let ns = timeout as u64;
        let timeout_tick = (ns * T::frequency().to_kHz() as u64).div_ceil(1_000_000);
        Self::new(timeout_tick)
    }

    pub fn new_us(timeout: u32) -> Self {
        let us = timeout as u64;
        let timeout_tick = (us * T::frequency().to_kHz() as u64).div_ceil(1_000);
        Self::new(timeout_tick)
    }

    pub fn new_ms(timeout: u32) -> Self {
        let ms = timeout as u64;
        let frequency = T::frequency().to_kHz() as u64;
        let timeout_tick = ms * frequency;
        Self::new(timeout_tick)
    }

    fn new(timeout_tick: u64) -> Self {
        Self {
            tick: T::now(),
            timeout_tick: timeout_tick,
            elapsed_tick: 0,
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
        let now = T::now();
        self.elapsed_tick = self
            .elapsed_tick
            .saturating_add(now.tick_since(self.tick) as u64);
        self.tick = now;

        if self.elapsed_tick >= self.timeout_tick {
            self.elapsed_tick -= self.timeout_tick;
            return true;
        }
        false
    }

    #[inline(always)]
    fn restart(&mut self) {
        self.tick = T::now();
        self.elapsed_tick = 0;
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

        fn tick_since(self, earlier: Self) -> u32 {
            self.tick.wrapping_sub(earlier.tick) as u32
        }
    }

    #[test]
    fn tick_timeout() {
        let now = MockInstant::now();
        assert_eq!(now.tick_elapsed(), 0);
        MockInstant::add_time(10.nanos());
        assert_eq!(now.tick_elapsed(), 10);
        MockInstant::add_time(1.millis());
        assert_eq!(now.tick_elapsed(), 1_000_010);
        MockInstant::add_time(10.micros());
        let now2 = MockInstant::now();
        assert_eq!(now2.tick_since(now), 1_010_010);
        MockInstant::add_time(10.micros());
        let now3 = MockInstant::now();
        assert_eq!(now3.tick_since(now2), 10_000);

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

        let t = TickTimeoutState::<MockInstant>::new_us(40_000_000);
        assert_eq!(t.timeout_tick, 40_000_000_000);

        let mut t = TickTimeoutState::<MockInstant>::new_ms(40_000);
        assert_eq!(t.timeout_tick, 40_000_000_000);

        assert!(!t.timeout());

        for _ in 0..40 {
            MockInstant::add_time(999.millis());
            assert!(!t.timeout());
        }
        assert!(t.elapsed_tick > 0);
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
