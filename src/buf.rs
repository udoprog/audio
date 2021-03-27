/// A trait describing an immutable audio buffer.
pub trait Buf<T> {
    /// The number of channels in the buffer.
    fn channels(&self) -> usize;

    /// Test if the given channel is masked.
    fn is_masked(&self, channel: usize) -> bool;

    /// Get the slice for any one given channel.
    ///
    /// # Panics
    ///
    /// Panics if the specified channel is out of bound.
    fn channel(&self, channel: usize) -> &[T];
}

/// The default vector of vectors buffer.
impl<T> Buf<T> for Vec<Vec<T>> {
    fn channels(&self) -> usize {
        self.len()
    }

    fn is_masked(&self, channel: usize) -> bool {
        self[channel].is_empty()
    }

    fn channel(&self, channel: usize) -> &[T] {
        &self[channel]
    }
}
