//! A library for writing audio to audio devices.
//!
//! You can regenerate the bindings for the library using:
//!
//! ```bash
//! cargo run --bin audio-device-bindings
//! ```

#[cfg(all(windows, feature = "wasapi"))]
pub mod wasapi;

#[cfg(windows)]
pub mod windows;

#[cfg(unix)]
pub mod unix;

#[cfg(feature = "alsa")]
pub mod libc;

#[cfg(all(unix, feature = "alsa"))]
pub mod alsa;

pub mod driver;

pub(crate) mod loom;
