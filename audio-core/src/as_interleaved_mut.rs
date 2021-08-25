use std::ptr;

/// A trait describing a buffer that is interleaved and mutable.
///
/// This allows for accessing the raw underlying interleaved buffer.
pub trait AsInterleavedMut<T> {
    /// Access the underlying interleaved mutable buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::{AsInterleaved, AsInterleavedMut, Buf, Channel};
    /// use audio::wrap;
    ///
    /// fn test(mut buffer: impl Buf<Sample = i16> + AsInterleaved<i16> + AsInterleavedMut<i16>) {
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
    fn as_interleaved_mut(&mut self) -> &mut [T];

    /// Access a pointer to the underlying interleaved mutable buffer.
    ///
    /// The length of the buffer is unspecified, unless preceded by a call to
    /// [reserve_frames]. Assuming the call doesn't panic, the pointed to buffer
    /// is guaranteed to be both allocated and initialized up until the number
    /// of frames as specified as argument to [reserve_frames].
    ///
    /// [reserve_frames]: crate::buf::Interleaved::reserve_frames
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::{AsInterleavedMut, Interleaved, Buf, Channel};
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
    /// fn test(mut buffer: impl Interleaved + AsInterleavedMut<i16>) {
    ///     buffer.reserve_frames(16);
    ///     // Note: call fills the buffer with ones.
    ///     // Safety: We've initialized exactly 16 frames before calling this
    ///     // function.
    ///     let (channels, frames) = unsafe { fill_with_ones(buffer.as_interleaved_mut_ptr(), 16) };
    ///     buffer.set_topology(channels, frames);
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
    fn as_interleaved_mut_ptr(&mut self) -> ptr::NonNull<T>;
}

impl<B, T> AsInterleavedMut<T> for &mut B
where
    B: ?Sized + AsInterleavedMut<T>,
{
    fn as_interleaved_mut(&mut self) -> &mut [T] {
        (**self).as_interleaved_mut()
    }

    fn as_interleaved_mut_ptr(&mut self) -> ptr::NonNull<T> {
        (**self).as_interleaved_mut_ptr()
    }
}
