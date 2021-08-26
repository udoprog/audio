use crate::Buf;

/// A trait describing a buffer that is interleaved.
///
/// This allows for accessing the raw underlying interleaved buffer.
pub trait InterleavedBuf: Buf {
    /// Access the underlying raw interleaved buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use audio::InterleavedBuf;
    ///
    /// fn test(buf: impl InterleavedBuf<Sample = i16>) {
    ///     assert_eq!(buf.as_interleaved(), &[1, 1, 2, 2, 3, 3, 4, 4]);
    /// }
    ///
    /// test(audio::interleaved![[1, 2, 3, 4]; 2]);
    /// ```
    fn as_interleaved(&self) -> &[Self::Sample];
}

impl<B> InterleavedBuf for &B
where
    B: ?Sized + InterleavedBuf,
{
    fn as_interleaved(&self) -> &[Self::Sample] {
        (**self).as_interleaved()
    }
}

impl<B> InterleavedBuf for &mut B
where
    B: ?Sized + InterleavedBuf,
{
    fn as_interleaved(&self) -> &[Self::Sample] {
        (**self).as_interleaved()
    }
}
