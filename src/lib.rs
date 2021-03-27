//! [![Documentation](https://docs.rs/rotary/badge.svg)](https://docs.rs/rotary)
//! [![Crates](https://img.shields.io/crates/v/rotary.svg)](https://crates.io/crates/rotary)
//! [![Actions Status](https://github.com/udoprog/rotary/workflows/Rust/badge.svg)](https://github.com/udoprog/rotary/actions)
//!
//! A library for dealing efficiently with AudioBuffer non-interleaved audio
//! buffers.
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
//! # Creating and using a masked audio buffer.
//!
//! ```rust
//! use rotary::BitSet;
//!
//! let mut buffer = rotary::MaskedAudioBuffer::<f32, BitSet<u128>>::with_topology(4, 1024);
//!
//! buffer.mask(1);
//!
//! for  channel in buffer.iter_mut() {
//!     for b in channel {
//!         *b = 1.0;
//!     }
//! }
//!
//! let expected = vec![1.0f32; 1024];
//!
//! assert_eq!(&buffer[0], &expected[..]);
//! assert_eq!(&buffer[1], &[][..]);
//! assert_eq!(&buffer[2], &expected[..]);
//! assert_eq!(&buffer[3], &expected[..]);
//! ```

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
