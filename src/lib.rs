//! [![Documentation](https://docs.rs/rotary/badge.svg)](https://docs.rs/rotary)
//! [![Crates](https://img.shields.io/crates/v/rotary.svg)](https://crates.io/crates/rotary)
//! [![Actions Status](https://github.com/udoprog/rotary/workflows/Rust/badge.svg)](https://github.com/udoprog/rotary/actions)
//!
//! A library for dealing efficiently with Dynamic non-interleaved audio
//! buffers.
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
//! You can use masked audio buffers if you need the ability to temporarily
//! enable or disable channels. This requires that you specify the type of the
//! mask. A good option for this is a [BitSet<u128>], which supports up to 128
//! channels.
//!
//! ```rust
//! use rotary::BitSet;
//!
//! let mut buffer = rotary::MaskedDynamic::<f32, BitSet<u128>>::with_topology(4, 128);
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
mod mask;
pub mod masked_dynamic;
pub mod range;
mod sample;
pub mod sequential;
#[cfg(test)]
mod tests;

pub use self::bit_set::BitSet;
pub use self::buf::{Buf, BufChannel, BufChannelMut, BufMut, MaskedBuf, MaskedBufMut};
pub use self::dynamic::Dynamic;
pub use self::interleaved::Interleaved;
pub use self::mask::Mask;
pub use self::masked_dynamic::MaskedDynamic;
pub use self::range::Range;
pub use self::sample::Sample;
pub use self::sequential::Sequential;
