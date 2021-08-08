use audio_core::{Buf, BufMut, ExactSizeBuf, LinearChannel, Slice, SliceMut};

/// A wrapper for a sequential audio buffer.
///
/// See [wrap::sequential][super::sequential()].
pub struct Sequential<T> {
    value: T,
    channels: usize,
}

impl<T> Sequential<T> {
    pub(super) fn new(value: T, channels: usize) -> Self {
        Self { value, channels }
    }

    /// Convert back into the wrapped value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let buffer = audio::wrap::sequential(&[1, 2, 3, 4], 2);
    /// assert_eq!(buffer.into_inner(), &[1, 2, 3, 4]);
    /// ```
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T> Buf for Sequential<T>
where
    T: Slice,
{
    type Sample = T::Item;
    type Channel<'a>
    where
        T::Item: 'a,
    = LinearChannel<&'a [T::Item]>;

    fn frames_hint(&self) -> Option<usize> {
        Some(self.frames())
    }

    fn channels(&self) -> usize {
        self.channels
    }

    fn channel(&self, channel: usize) -> Self::Channel<'_> {
        let frames = self.frames();
        let value = self
            .value
            .as_ref()
            .get(channel * frames..)
            .unwrap_or_default();
        let value = value.get(..frames).unwrap_or_default();
        LinearChannel::new(value)
    }
}

impl<T> ExactSizeBuf for Sequential<T>
where
    T: Slice,
{
    fn frames(&self) -> usize {
        self.value.as_ref().len() / self.channels
    }
}

impl<T> BufMut for Sequential<T>
where
    T: SliceMut,
{
    type ChannelMut<'a>
    where
        T::Item: 'a,
    = LinearChannel<&'a mut [T::Item]>;

    fn channel_mut(&mut self, channel: usize) -> Self::ChannelMut<'_> {
        let frames = self.frames();
        let value = self
            .value
            .as_mut()
            .get_mut(channel * frames..)
            .unwrap_or_default();
        let value = value.get_mut(..frames).unwrap_or_default();
        LinearChannel::new(value)
    }

    fn copy_channels(&mut self, from: usize, to: usize) {
        let frames = self.frames();

        // Safety: We're calling the copy function with internal
        // parameters which are guaranteed to be correct. `channels` is
        // guaranteed to reflect a valid subset of the buffer based on
        // frames, because it uses the trusted length of the provided
        // slice.
        unsafe {
            crate::utils::copy_channels_sequential(
                self.value.as_mut_ptr(),
                self.channels,
                frames,
                from,
                to,
            );
        }
    }
}
