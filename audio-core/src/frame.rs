//! A frame buffer as created through [Buf::get_frame][crate::Buf::get_frame] or
//! [BufMut::get_sample_mut][crate::BufMut::get_sample_mut].

/// The buffer of a single frame.
pub trait Frame {
    /// The sample of a channel.
    type Sample: Copy;

    /// The type the frame assumes when coerced into a reference.
    type Frame<'this>: Frame<Sample = Self::Sample>
    where
        Self: 'this;

    /// A borrowing iterator over the channel.
    type Iter<'this>: Iterator<Item = Self::Sample>
    where
        Self: 'this;

    /// Reborrow the current frame as a reference.
    fn as_frame(&self) -> Self::Frame<'_>;

    /// Get the length which indicates number of samples in the current frame.
    fn len(&self) -> usize;

    /// Test if the current frame is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the sample at the given channel in the frame.
    fn get(&self, channel: usize) -> Option<Self::Sample>;

    /// Construct an iterator over the frame.
    fn iter(&self) -> Self::Iter<'_>;
}
