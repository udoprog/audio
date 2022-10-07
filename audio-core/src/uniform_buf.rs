use crate::buf::Buf;
use crate::frame::Frame;

/// A buffer which has a unifom channel size.
pub trait UniformBuf: Buf {
    /// The type the channel assumes when coerced into a reference.
    type Frame<'this>: Frame<Sample = Self::Sample>
    where
        Self: 'this;

    /// A borrowing iterator over the channel.
    type FramesIter<'this>: Iterator<Item = Self::Sample>
    where
        Self: 'this;

    /// Get a single frame at the given offset.
    fn get_frame(&self, frame: usize) -> Option<Self::Frame<'_>>;

    /// Construct an iterator over all the frames in the audio buffer.
    fn iter_frames(&self) -> Self::FramesIter<'_>;
}
