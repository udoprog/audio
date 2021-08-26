use crate::Buf;

/// Trait implemented for buffers that can be resized.
pub trait ResizableBuf: Buf {
    /// Resize the number of per-channel frames in the buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::{Buf, ExactSizeBuf, ResizableBuf};
    ///
    /// fn test(mut buffer: impl ResizableBuf) {
    ///     buffer.resize(4);
    /// }
    ///
    /// let mut buffer = audio::interleaved![[0; 0]; 2];
    ///
    /// assert_eq!(buffer.channels(), 2);
    /// assert_eq!(buffer.frames(), 0);
    ///
    /// test(&mut buffer);
    ///
    /// assert_eq!(buffer.channels(), 2);
    /// assert_eq!(buffer.frames(), 4);
    /// ```
    fn resize(&mut self, frames: usize);

    /// Resize the buffer to match the given topology.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::{Buf, ExactSizeBuf, ResizableBuf};
    ///
    /// fn test(mut buffer: impl ResizableBuf) {
    ///     buffer.resize_topology(2, 4);
    /// }
    ///
    /// let mut buffer = audio::interleaved![[0; 0]; 4];
    ///
    /// test(&mut buffer);
    ///
    /// assert_eq!(buffer.channels(), 2);
    /// assert_eq!(buffer.frames(), 4);
    /// ```
    fn resize_topology(&mut self, channels: usize, frames: usize);
}

impl<B> ResizableBuf for &mut B
where
    B: ?Sized + ResizableBuf,
{
    fn resize(&mut self, frames: usize) {
        (**self).resize(frames);
    }

    fn resize_topology(&mut self, channels: usize, frames: usize) {
        (**self).resize_topology(channels, frames);
    }
}
