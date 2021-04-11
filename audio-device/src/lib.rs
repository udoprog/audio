//! A library for writing audio to audio devices.
//!
//! You can regenerate the bindings for the library using:
//!
//! ```bash
//! cargo run --bin audio-device-bindings
//! ```

#[cfg(all(windows, feature = "wasapi"))]
pub mod wasapi;

#[cfg(all(windows, feature = "xaudio2"))]
pub mod xaudio2;

#[cfg(windows)]
pub mod bindings;

#[cfg(windows)]
pub mod windows;

pub mod driver;

pub(crate) mod loom;
