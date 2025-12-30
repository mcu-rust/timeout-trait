# timeout-trait

[![CI](https://github.com/mcu-rust/timeout-trait/workflows/CI/badge.svg)](https://github.com/mcu-rust/timeout-trait/actions)
[![Crates.io](https://img.shields.io/crates/v/timeout-trait.svg)](https://crates.io/crates/timeout-trait)
[![Docs.rs](https://docs.rs/timeout-trait/badge.svg)](https://docs.rs/timeout-trait)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](./LICENSE)
[![Downloads](https://img.shields.io/crates/d/timeout-trait.svg)](https://crates.io/crates/timeout-trait)

Traits used to wait and timeout in a `no-std` embedded system.

It requires an implementation of `TickInstant`. In return, it provides `TickTimeout` and `TickDuration`, which can be used for timeout-related operations. It also includes an implementation of `DelayNs` called `TickDelay`, suitable for bare-metal systems.

For more details, see the [documentation](https://docs.rs/timeout-trait).


## Cargo Features

- `std`: Used for unit test. Disabled by default.
