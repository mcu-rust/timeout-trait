//! Traits used to wait and timeout in a `no-std` embedded system.
//!
//!
//! # Cargo Features
//!
//! - `std`: Used for unit test. Disabled by default.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod delay_impls;
pub mod duration;
pub mod fake_impls;
pub mod prelude;
#[cfg(feature = "std")]
pub mod std_impls;
pub mod tick_impl;

pub use delay_impls::TickDelay;
pub use embedded_hal;
pub use fake_impls::*;
pub use fugit::{self, KilohertzU32, MicrosDurationU32};
pub use tick_impl::{TickTimeoutNs, TickTimeoutState};

use duration::TickDuration;

pub trait TimeoutNs {
    type TimeoutState: TimeoutState;

    fn start_ns(&self, timeout: u32) -> Self::TimeoutState;
    fn start_us(&self, timeout: u32) -> Self::TimeoutState;
    fn start_ms(&self, timeout: u32) -> Self::TimeoutState;

    fn ns_with(&self, timeout: u32, mut f: impl FnMut() -> bool) -> bool {
        let mut t = self.start_ns(timeout);
        while f() {
            if t.timeout() {
                return true;
            }
        }
        false
    }

    fn us_with(&self, timeout: u32, mut f: impl FnMut() -> bool) -> bool {
        let mut t = self.start_us(timeout);
        while f() {
            if t.timeout() {
                return true;
            }
        }
        false
    }

    fn ms_with(&self, timeout: u32, mut f: impl FnMut() -> bool) -> bool {
        let mut t = self.start_ms(timeout);
        while f() {
            if t.timeout() {
                return true;
            }
        }
        false
    }
}

pub trait TimeoutState {
    /// Check if the time limit expires.
    fn timeout(&mut self) -> bool;
    /// Reset the timeout condition.
    fn restart(&mut self);
}

/// It doesn't require operation interfaces on `TickInstant` itself.
/// Embedded systems can thus implement only the relative time version.
pub trait TickInstant: Clone {
    fn frequency() -> KilohertzU32;
    fn now() -> Self;
    /// Returns the amount of ticks elapsed since this instant.
    fn elapsed(&mut self) -> TickDuration<Self>;
    #[must_use]
    fn add(&self, dur: &TickDuration<Self>) -> Self;

    #[inline]
    fn timeout(&mut self, dur: &TickDuration<Self>) -> bool {
        &self.elapsed() >= dur
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
