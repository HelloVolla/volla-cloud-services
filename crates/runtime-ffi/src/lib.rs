//! # holochain-conductor-runtime-ffi
//!
//! A simple wrapper around `holochain-conductor-runtime`,
//! with the addition of using [`uniffi`](https://docs.rs/uniffi/latest/uniffi/)
//! to generate FFI functions and Kotlin bindings to them.

// `RuntimeErrorFfi` wraps `RuntimeError`, which intentionally carries the full
// `AdminResponse` in `AdminApiBadResponse` for diagnostics; silence the size
// lint this trips on every `RuntimeResultFfi` return site.
#![allow(clippy::result_large_err)]

uniffi::setup_scaffolding!();

extern crate android_logger;
extern crate log;

mod autostart;
mod error;
mod multi_thread;
mod runtime;

pub use autostart::*;
pub use runtime::*;
