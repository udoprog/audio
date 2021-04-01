//! Wrap an external type to implement [Channels][crate::Channels] and
//! [ChannelsMut][crate::ChannelsMut].

mod interleaved;
pub use self::interleaved::Interleaved;

mod sequential;
pub use self::sequential::Sequential;

/// Wrap a `value` as an interleaved buffer with the given number of channels.
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
/// ```rust
/// use rotary::{wrap, io};
/// use rotary::ReadBuf as _;
///
/// let mut read_from = wrap::interleaved(&[0, 1, 2, 4, 5, 6, 7, 8][..], 2);
/// let mut write_to = io::Write::new(rotary::sequential![[0i16; 4]; 2]);
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
/// ```rust
/// use rotary::{wrap, io};
/// use rotary::WriteBuf as _;
///
/// let mut vec = vec![0, 1, 2, 4, 5, 6, 7, 8];
///
/// let mut read_from = io::Read::new(rotary::sequential![[0i16, 1i16, 2i16, 3i16]; 2]);
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
pub fn interleaved<T>(value: T, channels: usize) -> Interleaved<T> {
    Interleaved::new(value, channels)
}

/// Wrap a `value` as a sequential buffer with the given number of frames. The
/// length of the buffer determines the number of channels it has.
///
/// Unlike [interleaved][interleaved()], wrapped sequential buffers cannot be
/// used as implementations of [ReadBuf][crate::ReadBuf] or
/// [WriteBuf][crate::WriteBuf].
///
/// You can instead use the [Read][crate::io::Read] or [Write][crate::io::Write]
/// adapters available to accomplish this.
pub fn sequential<T>(value: T, channels: usize) -> Sequential<T> {
    Sequential::new(value, channels)
}
