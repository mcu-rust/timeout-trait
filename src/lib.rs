//! Traits used to wait and timeout in a `no-std` embedded system.
//!
//! It requires an implementation of [`TickInstant`]. In return, it provides [`TickTimeout`]
//! and [`TickDuration`], which can be used for timeout-related operations. It also includes
//! an implementation of `DelayNs` called [`TickDelay`], suitable for bare-metal systems.
//!
//! # Cargo Features
//!
//! - `std`: Used for unit test. Disabled by default.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod delay;
pub mod duration;
pub mod fake_impls;
pub mod prelude;
#[cfg(feature = "std")]
pub mod std_impls;
pub mod timeout;

#[cfg(all(feature = "std", test))]
mod mock;

pub use delay::TickDelay;
pub use duration::TickDuration;
pub use embedded_hal::delay::DelayNs;
pub use fugit::{self, KilohertzU32};
pub use timeout::TickTimeout;

/// It doesn't require operation interfaces on `TickInstant` itself.
/// Embedded systems can thus implement only the relative time version,
/// which means you can not use it as a global timestamp.
pub trait TickInstant: Clone {
    fn frequency() -> KilohertzU32;
    fn now() -> Self;
    /// Returns the amount of ticks elapsed since this instant.
    fn elapsed(&mut self) -> TickDuration<Self>;

    /// Move the instant forward, but it cannot be in the future.
    fn move_forward(&mut self, dur: &TickDuration<Self>);

    #[inline]
    fn timeout(&mut self, dur: &TickDuration<Self>) -> bool {
        &self.elapsed() >= dur
    }

    fn timeout_with(&self, dur: &TickDuration<Self>, mut f: impl FnMut() -> bool) -> bool {
        let mut t = Self::now();
        while f() {
            if t.timeout(dur) {
                return true;
            }
        }
        false
    }

    fn time_left(&mut self, dur: &TickDuration<Self>) -> TickDuration<Self> {
        let elapsed = &self.elapsed();
        if elapsed >= dur {
            TickDuration::ZERO
        } else {
            dur - elapsed
        }
    }
}
