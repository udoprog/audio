//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/audio-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/audio)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/audio.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/audio)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-audio-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/audio)
//! [<img alt="build status" src="https://img.shields.io/github/workflow/status/udoprog/audio/CI/main?style=for-the-badge" height="20">](https://github.com/udoprog/audio/actions?query=branch%3Amain)
//!
//! A crate for working with audio in Rust.
//!
//! This is made up of several parts, each can be used independently of each
//! other:
//!
//! * [audio-core] - The core crate, which defines traits that allows for
//!   interacting with audio buffers independent of their layout in memory.
//! * [audio] - This crate, which provides a collection of high-quality audio
//!   buffers which implements the traits provided in [audio-core].
//! * [audio-device] - A crate for interacting with audio devices in idiomatic
//!   Rust.
//! * [audio-generator] - A crate for generating audio.
//!
//! Audio buffers provided by this crate have zero or more channels that can be
//! iterated over. A channel is simply a sequence of samples. The samples within
//! each channel at one moment in time are a frame. A buffer can store channels
//! in various ways in memory, as detailed in the next section.
//!
//! <br>
//!
//! ## Buffers
//!
//! This crate provides several structs for storing buffers of multichannel audio.
//! The examples represent how the two channels `[1, 2, 3, 4]` and `[5, 6, 7, 8]`
//! are stored in memory:
//!
//! * [Dynamic]: each channel is stored in its own heap allocation.
//!   So `[1, 2, 3, 4]` and `[5, 6, 7, 8]`. This may be more performant when
//!   resizing freqently. Generally prefer one of the other buffer types for
//!   better CPU cache locality.
//! * [Interleaved]: samples of each channel are interleaved
//!   in one heap allocation. So `[1, 5, 2, 6, 3, 7, 4, 8]`.
//! * [Sequential]: each channel is stored one after the other
//!   in one heap allocation. So `[1, 2, 3, 4, 5, 6, 7, 8]`.
//!
//! These all implement the [Buf] and [BufMut] traits, allowing library authors
//! to abstract over any one specific format. The exact channel and frame count
//! of a buffer is known as its *topology*. The following example allocates
//! buffers with 4 frames and 2 channels. The buffers are arranged in memory
//! differently, but data is copied into them using the same API.
//!
//! ```rust
//! use audio::{BufMut, ChannelMut};
//!
//! let mut dynamic = audio::dynamic![[0i16; 4]; 2];
//! let mut interleaved = audio::interleaved![[0i16; 4]; 2];
//! let mut sequential = audio::sequential![[0i16; 4]; 2];
//!
//! audio::channel::copy_iter(0i16.., dynamic.get_mut(0).unwrap());
//! audio::channel::copy_iter(0i16.., interleaved.get_mut(0).unwrap());
//! audio::channel::copy_iter(0i16.., sequential.get_mut(0).unwrap());
//! ```
//!
//! We also support [wrapping][wrap] external buffers so that they can
//! interoperate like other audio buffers. Library authors using the [Buf]/[BufMut]
//! traits as generic inputs to functions should re-export the [wrap] module so
//! users of the library are not required to use this crate's buffer structs.
//!
//! <br>
//!
//! ## Example: [play-mp3]
//!
//! Play an mp3 file with [minimp3-rs], [cpal], and [rubato] for resampling.
//!
//! This example can handle with any channel and sample rate configuration.
//!
//! ```bash
//! cargo run --release --package audio-examples --bin play-mp3 -- path/to/file.mp3
//! ```
//!
//! <br>
//!
//! ## Examples
//!
//! ```rust
//! use rand::Rng;
//!
//! let mut buf = audio::buf::Dynamic::<f32>::new();
//!
//! buf.resize_channels(2);
//! buf.resize(2048);
//!
//! /// Fill both channels with random noise.
//! let mut rng = rand::thread_rng();
//! rng.fill(&mut buf[0]);
//! rng.fill(&mut buf[1]);
//! ```
//!
//! For convenience we also provide several macros for constructing various
//! forms of dynamic audio buffers. These should mostly be used for testing.
//!
//! ```rust
//! let mut buf = audio::buf::Dynamic::<f32>::with_topology(4, 8);
//!
//! for mut channel in &mut buf {
//!     for f in channel.iter_mut() {
//!         *f = 2.0;
//!     }
//! }
//!
//! assert_eq! {
//!     buf,
//!     audio::dynamic![[2.0; 8]; 4],
//! };
//!
//! assert_eq! {
//!     buf,
//!     audio::dynamic![[2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0]; 4],
//! };
//! ```
//!
//! [audio-core]: https://docs.rs/audio-core
//! [audio-device]: https://docs.rs/audio-device
//! [audio-generator]: https://docs.rs/audio-generator
//! [audio]: https://docs.rs/audio
//! [Buf]: https://docs.rs/audio-core/latest/audio_core/trait.Buf.html
//! [BufMut]: https://docs.rs/audio-core/latest/audio_core/trait.BufMut.html
//! [cpal]: https://github.com/RustAudio/cpal
//! [Dynamic::resize]: https://docs.rs/audio/latest/audio/dynamic/struct.Dynamic.html#method.resize
//! [dynamic!]: https://docs.rs/audio/latest/audio/macros/macro.dynamic.html
//! [Dynamic]: https://docs.rs/audio/latest/audio/dynamic/struct.Dynamic.html
//! [Interleaved]: https://docs.rs/audio/latest/audio/interleaved/struct.Interleaved.html
//! [minimp3-rs]: https://github.com/germangb/minimp3-rs
//! [play-mp3]: https://github.com/udoprog/audio/tree/main/examples/src/bin/play-mp3.rs
//! [rubato]: https://github.com/HEnquist/rubato
//! [Sequential]: https://docs.rs/audio/latest/audio/sequential/struct.Sequential.html
//! [wrap]: https://docs.rs/audio/latest/audio/wrap/index.html

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs, rustdoc::broken_intra_doc_links)]
#![allow(clippy::should_implement_trait)]

#[macro_use]
mod macros;
pub mod buf;
pub mod channel;
pub mod frame;
pub mod io;
pub mod slice;
mod utils;
pub mod wrap;

#[cfg(test)]
mod tests;

pub use audio_core::*;
