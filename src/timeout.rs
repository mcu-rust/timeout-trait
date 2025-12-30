use crate::*;

pub struct TickTimeout<T: TickInstant> {
    time: T,
    timeout: TickDuration<T>,
}

impl<T> TickTimeout<T>
where
    T: TickInstant,
{
    pub fn from_nanos(timeout: u32) -> Self {
        Self::new(TickDuration::from_nanos(timeout))
    }

    pub fn from_micros(timeout: u32) -> Self {
        Self::new(TickDuration::from_micros(timeout))
    }

    pub fn from_millis(timeout: u32) -> Self {
        Self::new(TickDuration::from_millis(timeout))
    }

    pub fn from_duration(timeout: &TickDuration<T>) -> Self {
        Self::new(timeout.clone())
    }

    fn new(timeout: TickDuration<T>) -> Self {
        Self {
            time: T::now(),
            timeout,
        }
    }

    /// Can be reused without calling `restart()`.
    #[inline]
    pub fn timeout(&mut self) -> bool {
        if self.time.timeout(&self.timeout) {
            self.time.move_forward(&self.timeout);
            return true;
        }
        false
    }

    #[inline(always)]
    pub fn restart(&mut self) {
        self.time = T::now();
    }

    #[inline]
    pub fn time_left(&mut self) -> TickDuration<T> {
        self.time.time_left(&self.timeout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::MockInstant;
    use fugit::ExtU32;

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

        let mut t = TickTimeout::<MockInstant>::from_nanos(100);
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

        let mut t = TickTimeout::<MockInstant>::from_micros(100);
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

        let mut t = TickTimeout::<MockInstant>::from_millis(100);
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
        let dur = TickDuration::<MockInstant>::from_nanos(100);
        let t = MockInstant::now();
        assert!(t.timeout_with(&dur, || {
            MockInstant::add_time(10.nanos());
            count += 1;
            true
        }));
        assert_eq!(count, 10);

        let t = TickTimeout::<MockInstant>::from_micros(40_000_000);
        assert_eq!(t.timeout.ticks(), 40_000_000_000);

        let mut t = TickTimeout::<MockInstant>::from_millis(40_000);
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
