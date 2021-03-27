//! [![Documentation](https://docs.rs/rotary/badge.svg)](https://docs.rs/rotary)
//! [![Crates](https://img.shields.io/crates/v/rotary.svg)](https://crates.io/crates/rotary)
//! [![Actions Status](https://github.com/udoprog/rotary/workflows/Rust/badge.svg)](https://github.com/udoprog/rotary/actions)
//!
//! A library for dealing efficiently with AudioBuffer non-interleaved audio
//! buffers.
//!
//! The buffer is constructed similarly to a `Vec<Vec<T>>`, except the interior
//! vector has a fixed size. And the buffer makes no attempt to clear data which
//! is freed when using functions such as [AudioBuffer::resize].
//!
//! ```rust
//! use rand::Rng as _;
//!
//! let mut buffer = rotary::AudioBuffer::<f32>::new();
//!
//! buffer.resize_channels(2);
//! buffer.resize(2048);
//!
//! /// Fill both channels with random noise.
//! let mut rng = rand::thread_rng();
//! rng.fill(&mut buffer[0]);
//! rng.fill(&mut buffer[1]);
//! ```
//!
//! You can use masked audio buffers if you need the ability to temporarily
//! enable or disable channels. This requires that you specify the type of the
//! mask. A good option for this is a [BitSet<u128>], which supports up to 128
//! channels.
//!
//! ```rust
//! use rotary::BitSet;
//!
//! let mut buffer = rotary::MaskedAudioBuffer::<f32, BitSet<u128>>::with_topology(4, 128);
//!
//! buffer.mask(1);
//!
//! for  channel in buffer.iter_mut() {
//!     for b in channel {
//!         *b = 1.0;
//!     }
//! }
//!
//! let expected = vec![1.0f32; 128];
//!
//! assert_eq!(&buffer[0], &expected[..]);
//! assert_eq!(&buffer[1], &[][..]);
//! assert_eq!(&buffer[2], &expected[..]);
//! assert_eq!(&buffer[3], &expected[..]);
//! ```
//!
//! For convenience we also provide the [audio_buffer!] macro when constructing
//! audio buffers.
//!
//! ```rust
//! use rotary::BitSet;
//!
//! let mut buf = rotary::AudioBuffer::<f32>::with_topology(4, 128);
//!
//! for channel in &mut buf {
//!     for f in channel {
//!         *f = 2.0;
//!     }
//! }
//!
//! assert_eq!(buf, rotary::audio_buffer![[2.0; 128]; 4])
//! ```
//!
//! [AudioBuffer::resize]: https://docs.rs/rotary/0/rotary/audio_buffer/struct.AudioBuffer.html#method.resize
//! [BitSet<u128>]: https://docs.rs/rotary/0/rotary/bit_set/struct.BitSet.html
//! [audio_buffer!]: https://docs.rs/rotary/0/rotary/macros/macro.audio_buffer.html

#![deny(missing_docs)]

#[macro_use]
mod macros;
pub mod audio_buffer;
pub mod bit_set;
mod mask;
pub mod masked_audio_buffer;
mod sample;
#[cfg(test)]
mod tests;

pub use self::audio_buffer::AudioBuffer;
pub use self::bit_set::BitSet;
pub use self::mask::Mask;
pub use self::masked_audio_buffer::MaskedAudioBuffer;
pub use self::sample::Sample;
