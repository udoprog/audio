//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/audio-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/audio)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/audio-device.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/audio-device)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-audio--device-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/audio-device)
//!
//! A library for interacting with audio devices.
//!
//! The sole aim of this crate is to provide idiomatic *low level* audio
//! interface drivers that can be used independently. If all you need is WASAPI
//! or ALSA, then that is all you pay for and you should have a decent
//! Rust-idiomatic programming experience.
//!
//! This is part of the [audio ecosystem] and makes use of core traits provided
//! by the [audio-core] crate.
//!
//! <br>
//!
//! ## Examples
//!
//! * [ALSA blocking playback][alsa-blocking].
//! * [ALSA async playback][alsa-async].
//! * [WASAPI blocking playback][wasapi-blocking].
//! * [WASAPI async playback][wasapi-async].
//!
//! <br>
//!
//! ## Support
//!
//! Supported tier 1 platforms and systems are the following:
//!
//! | Platform | System | Blocking | Async   |
//! |----------|--------|----------|---------|
//! | Windows  | WASAPI | **wip**  | **wip** |
//! | Linux    | ALSA   | **wip**  | **wip** |
//!
//! [audio ecosystem]: https://docs.rs/audio
//! [alsa-blocking]: https://github.com/udoprog/audio/blob/main/audio-device/examples/alsa.rs
//! [alsa-async]: https://github.com/udoprog/audio/blob/main/audio-device/examples/alsa-async.rs
//! [audio-core]: https://docs.rs/audio-core
//! [wasapi-async]: https://github.com/udoprog/audio/blob/main/audio-device/examples/wasapi-async.rs
//! [wasapi-blocking]: https://github.com/udoprog/audio/blob/main/audio-device/examples/wasapi.rs

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![no_std]

#[cfg(not(feature = "std"))]
compile_error!("audio-device: the 'std' feature is required");

#[cfg(not(feature = "alloc"))]
compile_error!("audio-device: the 'alloc' feature is required");

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

pub(crate) mod loom;

#[macro_use]
#[doc(hidden)]
mod macros;

#[cfg(feature = "unix")]
#[cfg_attr(docsrs, doc(cfg(feature = "unix")))]
#[macro_use]
pub mod unix;

#[cfg(feature = "wasapi")]
#[cfg_attr(docsrs, doc(cfg(feature = "wasapi")))]
pub mod wasapi;

#[cfg(any(feature = "windows"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "windows"))))]
pub mod windows;

#[cfg(feature = "libc")]
#[cfg_attr(docsrs, doc(cfg(feature = "libc")))]
pub mod libc;

#[cfg(feature = "alsa")]
#[cfg_attr(docsrs, doc(cfg(feature = "alsa")))]
pub mod alsa;

#[cfg(feature = "pulse")]
#[cfg_attr(docsrs, doc(cfg(feature = "pulse")))]
pub mod pulse;

#[cfg(feature = "pipewire")]
#[cfg_attr(docsrs, doc(cfg(feature = "pipewire")))]
pub mod pipewire;

pub mod runtime;

mod error;
pub use self::error::{Error, Result};
