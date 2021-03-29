//! [![Documentation](https://docs.rs/rotary/badge.svg)](https://docs.rs/rotary)
//! [![Crates](https://img.shields.io/crates/v/rotary.svg)](https://crates.io/crates/rotary)
//! [![Actions Status](https://github.com/udoprog/rotary/workflows/Rust/badge.svg)](https://github.com/udoprog/rotary/actions)
//!
//! A library for working with audio buffers
//!
//! The buffer is constructed similarly to a `Vec<Vec<T>>`, except the interior
//! vector has a fixed size. And the buffer makes no attempt to clear data which
//! is freed when using functions such as [Dynamic::resize].
//!
//! ```rust
//! use rand::Rng as _;
//!
//! let mut buffer = rotary::Dynamic::<f32>::new();
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
//! You can use the included [Mask] trait to separately keep track of active and
//! inactive channels in an audio buffer. This requires that you specify the
//! type of the mask. A good option for this is a [BitSet<u128>], which supports
//! up to 128 channels.
//!
//! ```rust
//! use rotary::{BitSet, Mask as _};
//!
//! let mut buffer = rotary::Dynamic::<f32>::with_topology(4, 1024);
//! let mask: rotary::BitSet<u128> = rotary::bit_set![0, 2, 3];
//!
//! for chan in mask.join(buffer.iter_mut()) {
//!     for b in chan {
//!         *b = 1.0;
//!     }
//! }
//!
//! let zeroed = vec![0.0f32; 1024];
//! let expected = vec![1.0f32; 1024];
//!
//! assert_eq!(&buffer[0], &expected[..]);
//! assert_eq!(&buffer[1], &zeroed[..]);
//! assert_eq!(&buffer[2], &expected[..]);
//! assert_eq!(&buffer[3], &expected[..]);
//! ```
//!
//! For convenience we also provide the [dynamic!] macro when constructing
//! audio buffers.
//!
//! ```rust
//! use rotary::BitSet;
//!
//! let mut buf = rotary::Dynamic::<f32>::with_topology(4, 128);
//!
//! for channel in &mut buf {
//!     for f in channel {
//!         *f = 2.0;
//!     }
//! }
//!
//! assert_eq!(buf, rotary::dynamic![[2.0; 128]; 4])
//! ```
//!
//! [Dynamic::resize]: https://docs.rs/rotary/0/rotary/dynamic/struct.Dynamic.html#method.resize
//! [BitSet<u128>]: https://docs.rs/rotary/0/rotary/bit_set/struct.BitSet.html
//! [dynamic!]: https://docs.rs/rotary/0/rotary/macros/macro.dynamic.html

#![deny(missing_docs)]

#[macro_use]
mod macros;
pub mod bit_set;
mod buf;
pub mod channel;
pub mod dynamic;
pub mod interleaved;
pub mod mask;
pub mod range;
mod sample;
pub mod sequential;
#[cfg(test)]
mod tests;
pub mod utils;
pub mod wrap;

pub use self::bit_set::BitSet;
pub use self::buf::{Buf, BufChannel, BufChannelMut, BufMut};
pub use self::dynamic::Dynamic;
pub use self::interleaved::Interleaved;
pub use self::mask::Mask;
pub use self::range::Range;
pub use self::sample::{Sample, Translate};
pub use self::sequential::Sequential;
