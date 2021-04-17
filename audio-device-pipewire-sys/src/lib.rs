//! [audio-device] system bindings for PulseAudio.
//!
//! These bindings are generated with:
//!
//! ```sh
//! cargo run --package generate --bin generate-pulse
//! ```
//!
//! [audio-device]: https://docs.rs/audio-device

#![allow(non_camel_case_types, non_upper_case_globals)]

use libc::{itimerspec, timespec};

include!("bindings.rs");
