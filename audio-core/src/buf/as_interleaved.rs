/// A trait describing a buffer that is interleaved.
///
/// This allows for accessing the raw underlying interleaved buffer.
pub trait AsInterleaved<T> {
    /// Access the underlying raw interleaved buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::AsInterleaved;
    ///
    /// fn test<B>(buffer: B) where B: AsInterleaved<i16> {
    ///     assert_eq!(buffer.as_interleaved(), &[1, 1, 2, 2, 3, 3, 4, 4]);
    /// }
    ///
    /// test(audio::interleaved![[1, 2, 3, 4]; 2]);
    /// ```
    fn as_interleaved(&self) -> &[T];
}

impl<B, T> AsInterleaved<T> for &B
where
    B: ?Sized + AsInterleaved<T>,
{
    fn as_interleaved(&self) -> &[T] {
        (**self).as_interleaved()
    }
}

impl<B, T> AsInterleaved<T> for &mut B
where
    B: ?Sized + AsInterleaved<T>,
{
    fn as_interleaved(&self) -> &[T] {
        (**self).as_interleaved()
    }
}
