use crate::{TickDuration, TickInstant};
use core::sync::atomic::{AtomicU64, Ordering};
use fugit::{KilohertzU32, NanosDurationU32, RateExtU32};

static TICK_SOURCE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy)]
pub struct MockInstant {
    pub(crate) tick: u64,
}

impl MockInstant {
    pub fn add_time(t: NanosDurationU32) {
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

    fn move_forward(&mut self, dur: &TickDuration<Self>) {
        self.tick += dur.as_ticks();
    }
}
