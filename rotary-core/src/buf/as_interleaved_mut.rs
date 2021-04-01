/// A trait describing a buffer that is interleaved and mutable.
///
/// This allows for accessing the raw underlying interleaved buffer.
pub trait AsInterleavedMut<T> {
    /// Access the underlying interleaved mutable buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::{Channels, AsInterleaved, AsInterleavedMut};
    /// use rotary::wrap;
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
    /// [reserve_frames]: crate::InterleavedBuf::reserve_frames
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::{AsInterleavedMut, InterleavedBuf, Channels};
    /// # unsafe fn fill_with_ones(buf: *mut i16, len: usize) -> (usize, usize) {
    /// #     let buf = std::slice::from_raw_parts_mut(buf, len);
    /// #
    /// #     for (o, b) in buf.iter_mut().zip(std::iter::repeat(1)) {
    /// #          *o = b;
    /// #     }
    /// #
    /// #     (2, len / 2)
    /// # }
    ///
    /// fn test<B>(mut buffer: B) where B: InterleavedBuf + AsInterleavedMut<i16> {
    ///     buffer.reserve_frames(16);
    ///     // Note: call fills the buffer with ones.
    ///     // Safety: We've initialized exactly 16 frames before calling this
    ///     // function.
    ///     let (channels, frames) = unsafe { fill_with_ones(buffer.as_interleaved_mut_ptr(), 16) };
    ///     buffer.set_topology(channels, frames);
    /// }
    ///
    /// let mut buf = rotary::Interleaved::new();
    /// test(&mut buf);
    ///
    /// assert_eq! {
    ///     buf.channel(0).iter().collect::<Vec<_>>(),
    ///     &[1, 1, 1, 1, 1, 1, 1, 1],
    /// };
    /// assert_eq! {
    ///     buf.channel(1).iter().collect::<Vec<_>>(),
    ///     &[1, 1, 1, 1, 1, 1, 1, 1],
    /// };
    /// ```
    fn as_interleaved_mut_ptr(&mut self) -> *mut T {
        self.as_interleaved_mut().as_mut_ptr()
    }
}

impl<B, T> AsInterleavedMut<T> for &mut B
where
    B: ?Sized + AsInterleavedMut<T>,
{
    fn as_interleaved_mut(&mut self) -> &mut [T] {
        (**self).as_interleaved_mut()
    }

    fn as_interleaved_mut_ptr(&mut self) -> *mut T {
        (**self).as_interleaved_mut_ptr()
    }
}
