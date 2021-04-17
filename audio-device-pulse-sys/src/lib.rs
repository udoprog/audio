//! [audio-device] system bindings for PulseAudio.
//!
//! These bindings are generated with:
//!
//! ```sh
//! cargo run --package generate --bin generate-pulse
//! ```
//!
//! [audio-device]: https://docs.rs/audio-device

#![allow(non_camel_case_types)]

use libc::{pollfd, timeval};

include!("bindings.rs");
