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
//! # Examples
//!
//! * [ALSA blocking playback][alsa-blocking].
//! * [ALSA async playback][alsa-async].
//! * [WASAPI blocking playback][wasapi-blocking].
//! * [WASAPI async playback][wasapi-async].
//!
//! # Support
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

cfg_unix! {
    pub mod libc;
}

cfg_alsa! {
    pub mod alsa;
}

pub mod runtime;

mod error;
pub use self::error::{Error, Result};
