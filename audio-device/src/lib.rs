//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/audio-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/audio)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/audio-device.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/audio-device)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-audio--device-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/audio-device)
//! [<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/udoprog/audio/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/udoprog/audio/actions?query=branch%3Amain)
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

pub(crate) mod loom;

#[macro_use]
#[doc(hidden)]
mod macros;

cfg_unix! {
    #[macro_use]
    pub mod unix;
}

cfg_wasapi! {
    pub mod wasapi;
}

cfg_windows! {
    pub mod windows;
}

cfg_libc! {
    pub mod libc;
}

cfg_alsa! {
    pub mod alsa;
}

cfg_pulse! {
    pub mod pulse;
}

cfg_pipewire! {
    pub mod pipewire;
}

pub mod runtime;

mod error;
pub use self::error::{Error, Result};
