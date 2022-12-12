//! This module provides wrappers to pass audio data from crates which use
//! different buffer formats into functions that take [Buf][crate::Buf] or
//! [BufMut][crate::BufMut] without needing to copy data into an intermediate
//! buffer. They may also be useful for incrementally introducing this crate
//! into a codebase that uses a different buffer format.

use crate::slice::Slice;

mod interleaved;
pub use self::interleaved::Interleaved;

mod sequential;
pub use self::sequential::Sequential;

#[cfg(feature = "std")]
mod dynamic;
#[cfg(feature = "std")]
pub use self::dynamic::Dynamic;

/// Wrap a slice as an interleaved buffer with the given number of channels.
///
/// Certain interleaved buffers can be used conveniently as implementors of
/// [ReadBuf][crate::ReadBuf] and [WriteBuf][crate::WriteBuf], due to the
/// convenient nature of the buffer living linearly in memory.
///
/// * `&[T]` - implements [ReadBuf][crate::ReadBuf].
/// * `&mut [T]` - implements [WriteBuf][crate::WriteBuf].
///
/// # Example using a buffer for linear I/O
///
/// ```
/// use audio::{wrap, io};
/// use audio::ReadBuf as _;
///
/// let mut read_from = wrap::interleaved(&[0, 1, 2, 4, 5, 6, 7, 8][..], 2);
/// let mut write_to = io::Write::new(audio::sequential![[0i16; 4]; 2]);
///
/// assert!(read_from.has_remaining());
/// io::copy_remaining(&mut read_from, &mut write_to);
/// assert!(!read_from.has_remaining());
///
/// assert_eq! {
///     write_to.as_ref().as_slice(),
///     &[0, 2, 5, 7, 1, 4, 6, 8],
/// };
/// ```
///
/// Or with a mutable slice for writing.
///
/// ```
/// use audio::{wrap, io};
/// use audio::WriteBuf as _;
///
/// let mut vec = vec![0, 1, 2, 4, 5, 6, 7, 8];
///
/// let mut read_from = io::Read::new(audio::sequential![[0i16, 1i16, 2i16, 3i16]; 2]);
/// let mut write_to = wrap::interleaved(&mut vec[..], 2);
///
/// assert!(write_to.has_remaining_mut());
/// io::copy_remaining(&mut read_from, &mut write_to);
/// assert!(!write_to.has_remaining_mut());
///
/// assert_eq! {
///     &vec[..],
///     &[0, 0, 1, 1, 2, 2, 3, 3],
/// };
/// ```
pub fn interleaved<T>(value: T, channels: usize) -> Interleaved<T>
where
    T: Slice,
{
    Interleaved::new(value, channels)
}

/// Wrap a slice as a sequential buffer with the given number of frames. The
/// length of the buffer determines the number of channels it has.
///
/// Unlike [interleaved][interleaved()], wrapped sequential buffers cannot be
/// used as implementations of [ReadBuf][crate::ReadBuf] or
/// [WriteBuf][crate::WriteBuf].
///
/// You can instead use the [Read][crate::io::Read] or [Write][crate::io::Write]
/// adapters available to accomplish this.
pub fn sequential<T>(value: T, channels: usize) -> Sequential<T>
where
    T: Slice,
{
    Sequential::new(value, channels)
}

/// Wrap a [Vec] of Vecs or slice of Vecs where each inner Vec is a channel.
/// The channels do not need to be equally sized.
///
/// This should only be used for external types which you have no control over
/// or if you legitimately need a buffer which doesn't have uniformly sized
/// channels. Otherwise you should prefer to use [Dynamic][crate::buf::Dynamic]
/// instead since it's more memory efficient.
///
/// # Example
///
/// ```
/// use audio::Buf;
///
/// let buf = vec![vec![1, 2, 3, 4], vec![5, 6]];
/// let buf = audio::wrap::dynamic(&buf[..]);
/// assert_eq!(buf.channels(), 2);
/// assert_eq!(buf.frames_hint(), Some(4));
/// ```
#[cfg(feature = "std")]
pub fn dynamic<T>(value: T) -> Dynamic<T> {
    Dynamic::new(value)
}
