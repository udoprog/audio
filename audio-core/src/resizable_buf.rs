/// Trait implemented for buffers that can be resized.
pub trait ResizableBuf {
    /// Ensure that the audio buffer has space for at least the given `capacity`
    /// of contiguous memory. The `capacity` is specified in number of
    /// [Samples][Buf::Sample].
    ///
    /// This is a no-op unless the underlying buffer is contiguous in memory
    /// which can be ensured by requiring traits such as
    /// [InterleavedBufMut][crate::InterleavedBufMut].
    ///
    /// This returns a boolean indicating if we could successfully reserve the
    /// given amount of memory. The caller can only assume that a buffer is
    /// present up to the given number of samples if it returns `true`. This is
    /// important to observe if the code you're working on has safety
    /// implications.
    ///
    /// A typical approach in case a reservation fails is to either panic or
    /// return an error indicating that the provided buffer is not supported.
    fn try_reserve(&mut self, capacity: usize) -> bool;

    /// Resize the number of per-channel frames in the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use audio::{Buf, ExactSizeBuf, ResizableBuf};
    ///
    /// fn test(mut buffer: impl ResizableBuf) {
    ///     buffer.resize(4);
    /// }
    ///
    /// let mut buf = audio::interleaved![[0; 0]; 2];
    ///
    /// assert_eq!(buf.channels(), 2);
    /// assert_eq!(buf.frames(), 0);
    ///
    /// test(&mut buf);
    ///
    /// assert_eq!(buf.channels(), 2);
    /// assert_eq!(buf.frames(), 4);
    /// ```
    fn resize(&mut self, frames: usize);

    /// Resize the buffer to match the given topology.
    ///
    /// # Examples
    ///
    /// ```
    /// use audio::ResizableBuf;
    ///
    /// fn test(mut buf: impl ResizableBuf) {
    ///     buf.resize_topology(2, 4);
    /// }
    ///
    /// let mut buf = audio::interleaved![[0; 0]; 4];
    ///
    /// test(&mut buf);
    ///
    /// assert_eq!(buf.channels(), 2);
    /// assert_eq!(buf.frames(), 4);
    /// ```
    fn resize_topology(&mut self, channels: usize, frames: usize);
}

impl<B> ResizableBuf for &mut B
where
    B: ?Sized + ResizableBuf,
{
    fn try_reserve(&mut self, capacity: usize) -> bool {
        (**self).try_reserve(capacity)
    }

    fn resize(&mut self, frames: usize) {
        (**self).resize(frames);
    }

    fn resize_topology(&mut self, channels: usize, frames: usize) {
        (**self).resize_topology(channels, frames);
    }
}
