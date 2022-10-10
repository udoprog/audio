//! A frame buffer as created through [Buf::get_frame][crate::Buf::get_frame] or
//! [BufMut::get_sample_mut][crate::BufMut::get_sample_mut].

use crate::frame::Frame;

/// The buffer of a single frame.
pub trait FrameMut: Frame {
    /// The type the frame assumes when coerced into a reference.
    type FrameMut<'this>: FrameMut<Sample = Self::Sample>
    where
        Self: 'this;

    /// A borrowing iterator over the channel.
    type IterMut<'this>: Iterator<Item = &'this mut Self::Sample>
    where
        Self: 'this;

    /// Reborrow the current frame as a reference.
    fn as_sample_mut(&self) -> Self::FrameMut<'_>;

    /// Get the sample mutable at the given channel in the frame.
    fn get_mut(&self, channel: usize) -> Option<&mut Self::Sample>;

    /// Construct a mutable iterator over the frame.
    fn iter_mut(&self) -> Self::IterMut<'_>;
}
