use super::{
    TickDuration,
    fugit::{KilohertzU32, RateExtU32},
    prelude::*,
    tick_impl::{TickTimeoutNs, TickTimeoutState},
};

pub type FakeTimeoutNs = TickTimeoutNs<FakeTickInstant>;
pub type FakeTimeoutState = TickTimeoutState<FakeTickInstant>;

#[derive(Clone)]
pub struct FakeTickInstant {
    count: u64,
}

impl TickInstant for FakeTickInstant {
    #[inline]
    fn frequency() -> KilohertzU32 {
        1.kHz()
    }

    #[inline]
    fn now() -> Self {
        Self { count: 0 }
    }

    #[inline]
    fn elapsed(&mut self) -> TickDuration<Self> {
        self.count = self.count.wrapping_add(1);
        TickDuration::from_ticks(self.count)
    }

    #[inline]
    fn add(&self, dur: &TickDuration<Self>) -> Self {
        Self {
            count: self.count.wrapping_add(dur.ticks()),
        }
    }
}
