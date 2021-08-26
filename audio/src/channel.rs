//! Channel buffers.
//!
//! * [LinearMut] and [LinearRef] wraps a mutable and immutable *linear* channel
//!   buffer respectively.
//! * [InterleavedMut] and [InterleavedRef] wraps mutable and immutable
//!   *interleaved* channel buffers respectively.

pub mod linear;
pub use self::linear::{LinearMut, LinearRef};

pub mod interleaved;
pub use self::interleaved::{InterleavedMut, InterleavedRef};
