//! Wrap an external type to implement [Buf][crate::Buf] and
//! [BufMut][crate::BufMut].

mod interleaved;
pub use self::interleaved::Interleaved;

mod sequential;
pub use self::sequential::Sequential;

/// Wrap a `value` as an interleaved buffer with the given number of channels.
///
/// An interleaved buffer is a bit special in that it can implement
/// [ReadBuf][crate::io::ReadBuf] and [WriteBuf][crate::io::WriteBuf] directly
/// if it wraps one of the following types:
/// * `&[T]` - Will implement [ReadBuf][crate::io::ReadBuf].
/// * `&mut [T]` - Will implement [WriteBuf][crate::io::WriteBuf].
pub fn interleaved<T>(value: T, channels: usize) -> Interleaved<T> {
    Interleaved::new(value, channels)
}

/// Wrap a `value` as a sequential buffer with the given number of frames. The
/// length of the buffer determines the number of channels.
pub fn sequential<T>(value: T, frames: usize) -> Sequential<T> {
    Sequential::new(value, frames)
}
