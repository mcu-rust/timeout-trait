use super::{
    TickDuration,
    fugit::{KilohertzU32, RateExtU32},
    prelude::*,
};

#[derive(Clone)]
pub struct FakeTickInstant {
    elapsed: u64,
}

impl TickInstant for FakeTickInstant {
    #[inline]
    fn frequency() -> KilohertzU32 {
        1000.MHz()
    }

    #[inline]
    fn now() -> Self {
        Self { elapsed: 0 }
    }

    #[inline]
    fn elapsed(&mut self) -> TickDuration<Self> {
        self.elapsed = self.elapsed.wrapping_add(1);
        TickDuration::from_ticks(self.elapsed)
    }

    #[inline]
    fn move_forward(&mut self, dur: &TickDuration<Self>) {
        self.elapsed = self.elapsed.saturating_sub(dur.ticks());
    }
}
