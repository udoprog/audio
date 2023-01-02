//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/audio-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/audio)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/audio-device-alsa-sys.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/audio-device-alsa-sys)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-audio--device--alsa--sys-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/audio-device-alsa-sys)
//! [<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/udoprog/audio/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/udoprog/audio/actions?query=branch%3Amain)
//!
//! [audio-device] system bindings for ALSA.
//!
//! These bindings are generated with:
//!
//! ```sh
//! cargo run --package generate --bin generate-alsa
//! ```
//!
//! [audio-device]: https://docs.rs/audio-device

#![allow(non_camel_case_types)]

use libc::{pid_t, pollfd, timespec, timeval, FILE};

pub const SND_PCM_NONBLOCK: ::std::os::raw::c_int = 0x1;
pub const SND_PCM_ASYNC: ::std::os::raw::c_int = 0x2;

pub const SND_SEQ_OPEN_OUTPUT: i32 = 1;
pub const SND_SEQ_OPEN_INPUT: i32 = 2;
pub const SND_SEQ_OPEN_DUPLEX: i32 = SND_SEQ_OPEN_OUTPUT | SND_SEQ_OPEN_INPUT;
pub const SND_SEQ_NONBLOCK: i32 = 0x0001;
pub const SND_SEQ_ADDRESS_BROADCAST: u8 = 255;
pub const SND_SEQ_ADDRESS_SUBSCRIBERS: u8 = 254;
pub const SND_SEQ_ADDRESS_UNKNOWN: u8 = 253;
pub const SND_SEQ_QUEUE_DIRECT: u8 = 253;
pub const SND_SEQ_TIME_MODE_MASK: u8 = 1 << 1;
pub const SND_SEQ_TIME_STAMP_MASK: u8 = 1 << 0;
pub const SND_SEQ_TIME_MODE_REL: u8 = 1 << 1;
pub const SND_SEQ_TIME_STAMP_REAL: u8 = 1 << 0;
pub const SND_SEQ_TIME_STAMP_TICK: u8 = 0 << 0;
pub const SND_SEQ_TIME_MODE_ABS: u8 = 0 << 1;
pub const SND_SEQ_CLIENT_SYSTEM: u8 = 0;
pub const SND_SEQ_PORT_SYSTEM_TIMER: u8 = 0;
pub const SND_SEQ_PORT_SYSTEM_ANNOUNCE: u8 = 1;
pub const SND_SEQ_PRIORITY_HIGH: u8 = 1 << 4;
pub const SND_SEQ_EVENT_LENGTH_FIXED: u8 = 0 << 2;
pub const SND_SEQ_EVENT_LENGTH_MASK: u8 = 3 << 2;
pub const SND_SEQ_EVENT_LENGTH_VARIABLE: u8 = 1 << 2;
pub const SND_SEQ_EVENT_LENGTH_VARUSR: u8 = 2 << 2;

include!("bindings.rs");
