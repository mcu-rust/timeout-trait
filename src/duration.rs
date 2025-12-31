use crate::*;
use core::{cmp::Ordering, marker::PhantomData, ops};

#[derive(Clone, Copy)]
pub struct TickDuration<T: TickInstant> {
    ticks: u64,
    _t: PhantomData<T>,
}

impl<T: TickInstant> TickDuration<T> {
    pub const ZERO: Self = Self::from_ticks(0);
    pub const MAX: Self = Self::from_ticks(u64::MAX);

    #[inline]
    pub const fn from_ticks(ticks: u64) -> Self {
        Self {
            ticks,
            _t: PhantomData,
        }
    }

    #[inline]
    pub const fn as_ticks(&self) -> u64 {
        self.ticks
    }

    pub fn nanos(timeout: u32) -> Self {
        let ns = timeout as u64;
        Self::from_ticks((ns * T::frequency().to_kHz() as u64).div_ceil(1_000_000))
    }

    pub fn as_nanos(&self) -> u32 {
        if let Some(t) = self.ticks.checked_mul(1_000_000) {
            let rst = t.div_ceil(T::frequency().to_kHz() as u64);
            if rst <= u32::MAX as u64 {
                return rst as u32;
            }
        }
        panic!();
    }

    pub fn micros(timeout: u32) -> Self {
        let us = timeout as u64;
        Self::from_ticks((us * T::frequency().to_kHz() as u64).div_ceil(1_000))
    }

    pub fn as_micros(&self) -> u32 {
        if let Some(t) = self.ticks.checked_mul(1_000) {
            let rst = t.div_ceil(T::frequency().to_kHz() as u64);
            if rst <= u32::MAX as u64 {
                return rst as u32;
            }
        }
        panic!();
    }

    pub fn millis(timeout: u32) -> Self {
        let ms = timeout as u64;
        Self::from_ticks(ms * T::frequency().to_kHz() as u64)
    }

    pub fn as_millis(&self) -> u32 {
        let rst = self.ticks.div_ceil(T::frequency().to_kHz() as u64);
        if rst <= u32::MAX as u64 {
            return rst as u32;
        }
        panic!();
    }

    #[inline]
    pub const fn is_zero(&self) -> bool {
        self.ticks == 0
    }
}

impl<T: TickInstant> PartialEq<Self> for TickDuration<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.ticks.eq(&other.ticks)
    }
}

impl<T: TickInstant> PartialOrd<Self> for TickDuration<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.ticks.partial_cmp(&other.ticks)
    }
}

impl<T: TickInstant> ops::Add<Self> for &TickDuration<T> {
    type Output = TickDuration<T>;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        TickDuration::from_ticks(self.ticks.saturating_add(rhs.ticks))
    }
}

impl<T: TickInstant> ops::Sub<Self> for &TickDuration<T> {
    type Output = TickDuration<T>;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        TickDuration::from_ticks(self.ticks.saturating_sub(rhs.ticks))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::MockInstant;
    type Duration = TickDuration<MockInstant>;

    #[test]
    fn duration() {
        let d = Duration::nanos(123);
        assert_eq!(d.as_nanos(), 123);

        let d = Duration::nanos(123_000);
        assert_eq!(d.as_nanos(), 123_000);

        let d = Duration::micros(5234);
        assert_eq!(d.as_micros(), 5234);

        let d = Duration::millis(472);
        assert_eq!(d.as_millis(), 472);

        let d = Duration::micros(123);
        assert_eq!(d.as_millis(), 1);
    }
}
