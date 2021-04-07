//! A library for writing audio to audio devices.
//!
//! You can regenerate the bindings for the library using:
//!
//! ```bash
//! cargo run --bin rotary-device-bindings
//! ```

#[cfg(all(windows, feature = "wasapi"))]
pub mod wasapi;

#[cfg(all(windows, feature = "xaudio2"))]
pub mod xaudio2;

#[cfg(windows)]
pub mod bindings;

#[cfg(windows)]
mod windows;

mod audio_thread;
pub use self::audio_thread::{AudioThread, Panicked};
