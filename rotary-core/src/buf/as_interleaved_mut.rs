/// A trait describing a buffer that is interleaved and mutable.
///
/// This allows for accessing the raw underlying interleaved buffer.
pub trait AsInterleavedMut<T> {
    /// Access the underlying raw interleaved mutable buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::{Channels, AsInterleaved, AsInterleavedMut};
    ///
    /// fn test<B>(mut buffer: B) where B: Channels<i16> + AsInterleaved<i16> + AsInterleavedMut<i16> {
    ///     buffer.as_interleaved_mut().copy_from_slice(&[1, 1, 2, 2, 3, 3, 4, 4]);
    ///
    ///     assert_eq! {
    ///         buffer.channel(0).iter().collect::<Vec<_>>(),
    ///         &[1, 2, 3, 4],
    ///     };
    ///
    ///     assert_eq! {
    ///         buffer.channel(1).iter().collect::<Vec<_>>(),
    ///         &[1, 2, 3, 4],
    ///     };
    ///
    ///     assert_eq!(buffer.as_interleaved(), &[1, 1, 2, 2, 3, 3, 4, 4]);
    /// }
    ///
    /// test(rotary::interleaved![[0; 4]; 2]);
    /// ```
    fn as_interleaved_mut(&mut self) -> &mut [T];
}

impl<B, T> AsInterleavedMut<T> for &mut B
where
    B: ?Sized + AsInterleavedMut<T>,
{
    fn as_interleaved_mut(&mut self) -> &mut [T] {
        (**self).as_interleaved_mut()
    }
}
