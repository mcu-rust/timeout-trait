use super::{
    fugit::{KilohertzU32, RateExtU32},
    prelude::*,
    tick_impl::{TickTimeoutNs, TickTimeoutState},
};
use core::sync::atomic::{AtomicU32, Ordering};

pub type FakeTimeoutNs = TickTimeoutNs<FakeInstant>;
pub type FakeTimeoutState = TickTimeoutState<FakeInstant>;

static COUNTER: AtomicU32 = AtomicU32::new(0);

#[derive(Clone, Copy)]
pub struct FakeInstant {
    count: u32,
}

impl TickInstant for FakeInstant {
    fn frequency() -> KilohertzU32 {
        1.kHz()
    }

    fn now() -> Self {
        COUNTER.fetch_add(1, Ordering::Relaxed);
        Self {
            count: COUNTER.load(Ordering::Relaxed),
        }
    }

    fn tick_since(self, earlier: Self) -> u32 {
        self.count.wrapping_sub(earlier.count)
    }
}
