use crate::InterleavedBuf;
use std::ptr;

/// A trait describing a buffer that is interleaved and mutable.
///
/// This allows for accessing the raw underlying interleaved buffer.
pub trait InterleavedBufMut: InterleavedBuf {
    /// Access the underlying interleaved mutable buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use audio::{InterleavedBuf, InterleavedBufMut, Buf, Channel};
    /// use audio::wrap;
    ///
    /// fn test(mut buffer: impl Buf<Sample = i16> + InterleavedBufMut<Sample = i16>) {
    ///     buffer.as_interleaved_mut().copy_from_slice(&[1, 1, 2, 2, 3, 3, 4, 4]);
    ///
    ///     assert_eq! {
    ///         buffer.get(0).unwrap().iter().collect::<Vec<_>>(),
    ///         &[1, 2, 3, 4],
    ///     };
    ///
    ///     assert_eq! {
    ///         buffer.get(1).unwrap().iter().collect::<Vec<_>>(),
    ///         &[1, 2, 3, 4],
    ///     };
    ///
    ///     assert_eq!(buffer.as_interleaved(), &[1, 1, 2, 2, 3, 3, 4, 4]);
    /// }
    ///
    /// test(audio::interleaved![[0; 4]; 2]);
    /// let mut buf = [0; 8];
    /// test(wrap::interleaved(&mut buf, 2));
    /// ```
    fn as_interleaved_mut(&mut self) -> &mut [Self::Sample];

    /// Access a pointer to the underlying interleaved mutable buffer.
    ///
    /// The length of the buffer is unspecified, unless preceded by a call to
    /// [try_reserve]. Assuming the call doesn't panic, the pointed to buffer is
    /// guaranteed to be both allocated and initialized up until the number of
    /// frames as specified as argument to [try_reserve].
    ///
    /// [try_reserve]: crate::ResizableBuf::try_reserve
    ///
    /// # Examples
    ///
    /// ```
    /// use audio::{Buf, Channel, ResizableBuf, InterleavedBufMut};
    /// # unsafe fn fill_with_ones(buf: std::ptr::NonNull<i16>, len: usize) -> (usize, usize) {
    /// #     let buf = std::slice::from_raw_parts_mut(buf.as_ptr(), len);
    /// #
    /// #     for (o, b) in buf.iter_mut().zip(std::iter::repeat(1)) {
    /// #          *o = b;
    /// #     }
    /// #
    /// #     (2, len / 2)
    /// # }
    ///
    /// fn test(mut buffer: impl InterleavedBufMut<Sample = i16> + ResizableBuf) {
    ///     assert!(buffer.try_reserve(16));
    ///     // Note: call fills the buffer with ones.
    ///     // Safety: We've initialized exactly 16 frames before calling this
    ///     // function.
    ///     unsafe {
    ///         let (channels, frames) = fill_with_ones(buffer.as_interleaved_mut_ptr(), 16);
    ///         buffer.set_interleaved_topology(channels, frames);
    ///     }
    /// }
    ///
    /// let mut buf = audio::buf::Interleaved::new();
    /// test(&mut buf);
    ///
    /// assert_eq! {
    ///     buf.get(0).unwrap().iter().collect::<Vec<_>>(),
    ///     &[1, 1, 1, 1, 1, 1, 1, 1],
    /// };
    /// assert_eq! {
    ///     buf.get(1).unwrap().iter().collect::<Vec<_>>(),
    ///     &[1, 1, 1, 1, 1, 1, 1, 1],
    /// };
    /// ```
    fn as_interleaved_mut_ptr(&mut self) -> ptr::NonNull<Self::Sample>;

    /// Specify the topology of the underlying interleaved buffer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the topology of the underlying buffer has
    /// been updated to match the specified parameters.
    ///
    /// # Examples
    ///
    /// ```
    /// use audio::{Buf, Channel, ResizableBuf, InterleavedBufMut};
    /// # unsafe fn fill_with_ones(buf: std::ptr::NonNull<i16>, len: usize) -> (usize, usize) {
    /// #     let buf = std::slice::from_raw_parts_mut(buf.as_ptr(), len);
    /// #
    /// #     for (o, b) in buf.iter_mut().zip(std::iter::repeat(1)) {
    /// #          *o = b;
    /// #     }
    /// #
    /// #     (2, len / 2)
    /// # }
    ///
    /// fn test(mut buffer: impl InterleavedBufMut<Sample = i16> + ResizableBuf) {
    ///     assert!(buffer.try_reserve(16));
    ///     // Note: call fills the buffer with ones.
    ///     // Safety: We've initialized exactly 16 frames before calling this
    ///     // function.
    ///     unsafe {
    ///         let (channels, frames) = fill_with_ones(buffer.as_interleaved_mut_ptr(), 16);
    ///         buffer.set_interleaved_topology(channels, frames);
    ///     };
    /// }
    ///
    /// let mut buf = audio::buf::Interleaved::new();
    /// test(&mut buf);
    ///
    /// assert_eq! {
    ///     buf.get(0).unwrap().iter().collect::<Vec<_>>(),
    ///     &[1, 1, 1, 1, 1, 1, 1, 1],
    /// };
    /// assert_eq! {
    ///     buf.get(1).unwrap().iter().collect::<Vec<_>>(),
    ///     &[1, 1, 1, 1, 1, 1, 1, 1],
    /// };
    /// ```
    unsafe fn set_interleaved_topology(&mut self, channels: usize, frames: usize);
}

impl<B> InterleavedBufMut for &mut B
where
    B: ?Sized + InterleavedBufMut,
{
    #[inline]
    fn as_interleaved_mut(&mut self) -> &mut [Self::Sample] {
        (**self).as_interleaved_mut()
    }

    #[inline]
    fn as_interleaved_mut_ptr(&mut self) -> ptr::NonNull<Self::Sample> {
        (**self).as_interleaved_mut_ptr()
    }

    #[inline]
    unsafe fn set_interleaved_topology(&mut self, channels: usize, frames: usize) {
        (**self).set_interleaved_topology(channels, frames);
    }
}
