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
    pub const fn ticks(&self) -> u64 {
        self.ticks
    }

    pub fn from_nanos(timeout: u32) -> Self {
        let ns = timeout as u64;
        Self::from_ticks((ns * T::frequency().to_kHz() as u64).div_ceil(1_000_000))
    }

    pub fn as_nanos(&self) -> u32 {
        self.ticks
            .saturating_mul(1_000_000)
            .div_ceil(T::frequency().to_kHz() as u64) as u32
    }

    pub fn from_micros(timeout: u32) -> Self {
        let us = timeout as u64;
        Self::from_ticks((us * T::frequency().to_kHz() as u64).div_ceil(1_000))
    }

    pub fn as_micros(&self) -> u32 {
        self.ticks
            .saturating_mul(1_000)
            .div_ceil(T::frequency().to_kHz() as u64) as u32
    }

    pub fn from_millis(timeout: u32) -> Self {
        let ms = timeout as u64;
        Self::from_ticks(ms * T::frequency().to_kHz() as u64)
    }

    pub fn as_millis(&self) -> u32 {
        self.ticks.div_ceil(T::frequency().to_kHz() as u64) as u32
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
        let d = Duration::from_nanos(123);
        assert_eq!(d.as_nanos(), 123);

        let d = Duration::from_nanos(123_000);
        assert_eq!(d.as_nanos(), 123_000);

        let d = Duration::from_micros(5234);
        assert_eq!(d.as_micros(), 5234);

        let d = Duration::from_millis(472);
        assert_eq!(d.as_millis(), 472);

        let d = Duration::from_micros(123);
        assert_eq!(d.as_millis(), 1);
    }
}
