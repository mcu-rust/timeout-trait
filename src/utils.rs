use super::fugit::KilohertzU32;
use core::cell::Cell;

/// Can be used as a static holder.
/// You can use `AtomicU32` or `AtomicCell` instead.
pub struct FrequencyHolder {
    frequency: Cell<KilohertzU32>,
}

unsafe impl Sync for FrequencyHolder {}

impl FrequencyHolder {
    pub const fn new(frequency: KilohertzU32) -> Self {
        Self {
            frequency: Cell::new(frequency),
        }
    }

    pub fn set(&self, frequency: KilohertzU32) {
        critical_section::with(|_| {
            self.frequency.set(frequency);
        })
    }

    pub fn get(&self) -> KilohertzU32 {
        self.frequency.get()
    }
}
