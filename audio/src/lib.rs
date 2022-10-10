//! [![Documentation](https://docs.rs/audio/badge.svg)](https://docs.rs/audio)
//! [![Crates](https://img.shields.io/crates/v/audio.svg)](https://crates.io/crates/audio)
//! [![Actions Status](https://github.com/udoprog/audio/workflows/Rust/badge.svg)](https://github.com/udoprog/audio/actions)
//!
//! A crate for working with audio in Rust.
//!
//! This is made up of several parts, each can be used independently of each other:
//!
//! * [audio-core] - The core crate, which defines traits that allows for safely
//!   interacting with audio buffers.
//! * [audio] - This crate, which provides a collection of high-quality audio
//!   buffers which implements the traits provided in [audio-core].
//! * [audio-device] - A crate for interacting with audio devices in idiomatic
//!   Rust.
//! * [audio-generator] - A crate for generating audio.
//!
//! Audio buffers provided by this crate are conceptually kinda like
//! `Vec<Vec<T>>`, except the interior vector has a fixed size. And the buffer
//! makes no attempt to *clear data* which is freed when using resizing
//! functions such as [Dynamic::resize].
//!
//! # Formats and topologies
//!
//! The following are the three canonical audio formats which are supported by
//! this crate:
//! * [dynamic][Dynamic] - where each channel is stored in its own
//!   heap-allocated buffer.
//! * [interleaved][Interleaved] - where each channel is interleaved, like
//!   `0:0, 1:0, 1:0, 1:1`.
//! * [sequential][Sequential] - where each channel is stored in a linear
//!   buffer, one after another. Like `0:0, 0:1, 1:0, 1:0`.
//!
//! These all implement the [Buf] and [BufMut] traits, allowing library authors
//! to abstract over any one specific format. The exact channel and frame count
//! of a buffer is known as its *topology*.
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
//! interoperate like other audio buffers.
//!
//! # Example: [play-mp3]
//!
//! Play an mp3 file with [minimp3-rs], [cpal], and [rubato] for resampling.
//!
//! This example can handle with any channel and sample rate configuration.
//!
//! ```bash
//! cargo run --release --package audio-examples --bin play-mp3 -- path/to/file.mp3
//! ```
//!
//! # Examples
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
//! [Buf]: https://docs.rs/audio-core/*/audio_core/trait.Buf.html
//! [BufMut]: https://docs.rs/audio-core/*/audio_core/trait.BufMut.html
//! [cpal]: https://github.com/RustAudio/cpal
//! [Dynamic::resize]: https://docs.rs/audio/*/audio/dynamic/struct.Dynamic.html#method.resize
//! [dynamic!]: https://docs.rs/audio/*/audio/macros/macro.dynamic.html
//! [Dynamic]: https://docs.rs/audio/*/audio/dynamic/struct.Dynamic.html
//! [Interleaved]: https://docs.rs/audio/*/audio/interleaved/struct.Interleaved.html
//! [minimp3-rs]: https://github.com/germangb/minimp3-rs
//! [play-mp3]: https://github.com/udoprog/audio/tree/main/examples/src/bin/play-mp3.rs
//! [rubato]: https://github.com/HEnquist/rubato
//! [Sequential]: https://docs.rs/audio/*/audio/sequential/struct.Sequential.html
//! [wrap]: https://docs.rs/audio/*/audio/wrap/index.html

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
