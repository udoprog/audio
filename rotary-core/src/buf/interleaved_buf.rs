/// Trait implemented for buffers which are linearly stored in memory.
///
/// This allows us to reason about a specific number of frames, so that the
/// buffer can safely make sure that a given required number of frames are
/// available in the buffer, and it allows us to access the underlying buffer.
///
/// This is usually used in combination with other traits, such as
/// [AsInterleaved] to allow for generically accessing a fixed linear buffer
/// with a specific topology.
pub trait InterleavedBuf {
    /// Make sure that the buffer has reserved exactly the given number of
    /// frames.
    ///
    /// # Panics
    ///
    /// Panics if the underlying buffer cannot be grown to accomodate the given
    /// number of frames.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::{ExactSizeBuf as _, Buf as _, InterleavedBuf};
    ///
    /// fn test<B>(mut buffer: B) where B: InterleavedBuf {
    ///     buffer.reserve_frames(4);
    ///     buffer.set_topology(2, 2);
    /// }
    ///
    /// // Note: a zero-sized buffer.
    /// let mut buffer = rotary::interleaved![[0; 0]; 0];
    /// test(&mut buffer);
    ///
    /// assert_eq!(buffer.channels(), 2);
    /// assert_eq!(buffer.frames(), 2);
    /// assert_eq!(buffer.as_slice(), &[0, 0, 0, 0]);
    ///
    /// let mut buffer = [1, 2, 3, 4, 5, 6, 7, 8];
    /// let mut buffer = rotary::wrap::interleaved(&mut buffer[..], 1);
    /// test(&mut buffer);
    ///
    /// assert_eq!(buffer.channels(), 2);
    /// assert_eq!(buffer.frames(), 2);
    /// assert_eq!(buffer.into_inner(), &[1, 2, 3, 4]);
    /// ```
    fn reserve_frames(&mut self, frames: usize);

    /// Set the buffer to have the given topology.
    ///
    /// This function will never reallocate the underlying buffer.
    ///
    /// # Panics
    ///
    /// Calling [set_topology] will fail if the underlying buffer doesn't
    /// support the specified topology. Like if it's too small.
    ///
    /// If you want to use a growable buffer use
    /// [ResizableBuf::resize_topology][crate::ResizableBuf::resize_topology]
    /// instead.
    ///
    /// ```rust,should_panic
    /// use rotary::{ExactSizeBuf as _, Buf as _, InterleavedBuf};
    ///
    /// fn test<B>(mut buffer: B) where B: InterleavedBuf {
    ///     buffer.set_topology(2, 4); // panics because buffer is zero-sized.
    /// }
    ///
    /// test(rotary::interleaved![[0; 0]; 4]);
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::{ExactSizeBuf as _, Buf as _, InterleavedBuf};
    ///
    /// fn test<B>(mut buffer: B) where B: InterleavedBuf {
    ///     buffer.set_topology(2, 4);
    /// }
    ///
    /// // Note: a zero-sized buffer.
    /// let mut buffer = rotary::interleaved![[4; 2]; 4];
    /// assert_eq!(buffer.channels(), 4);
    /// assert_eq!(buffer.frames(), 2);
    ///
    /// test(&mut buffer);
    ///
    /// assert_eq!(buffer.channels(), 2);
    /// assert_eq!(buffer.frames(), 4);
    /// ```
    fn set_topology(&mut self, channels: usize, frames: usize);
}

impl<B> InterleavedBuf for &mut B
where
    B: ?Sized + InterleavedBuf,
{
    fn reserve_frames(&mut self, frames: usize) {
        (**self).reserve_frames(frames);
    }

    fn set_topology(&mut self, channels: usize, frames: usize) {
        (**self).set_topology(channels, frames);
    }
}
