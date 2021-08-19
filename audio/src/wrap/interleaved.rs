use audio_core::{
    AsInterleaved, AsInterleavedMut, Buf, BufMut, ExactSizeBuf, InterleavedBuf, InterleavedChannel,
    InterleavedChannelMut, ReadBuf, Slice, SliceMut, WriteBuf,
};

/// A wrapper for an interleaved audio buffer.
///
/// See [wrap::interleaved][super::interleaved()].
pub struct Interleaved<T> {
    value: T,
    channels: usize,
}

impl<T> Interleaved<T> {
    pub(super) fn new(value: T, channels: usize) -> Self {
        Self { value, channels }
    }

    /// Convert back into the wrapped value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let buffer = audio::wrap::interleaved(&[1, 2, 3, 4], 2);
    /// assert_eq!(buffer.into_inner(), &[1, 2, 3, 4]);
    /// ```
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T> Buf for Interleaved<T>
where
    T: Slice,
{
    type Sample = T::Item;
    type Channel<'a>
    where
        T::Item: 'a,
    = InterleavedChannel<'a, T::Item>;

    fn frames_hint(&self) -> Option<usize> {
        Some(self.frames())
    }

    fn channels(&self) -> usize {
        self.channels
    }

    fn channel(&self, channel: usize) -> Self::Channel<'_> {
        InterleavedChannel::from_slice(self.value.as_ref(), channel, self.channels)
    }
}

impl<T> ExactSizeBuf for Interleaved<T>
where
    T: Slice,
{
    fn frames(&self) -> usize {
        self.value.as_ref().len() / self.channels
    }
}

impl<T> AsInterleaved<T::Item> for Interleaved<T>
where
    T: Slice,
{
    fn as_interleaved(&self) -> &[T::Item] {
        self.value.as_ref()
    }
}

impl<T> BufMut for Interleaved<T>
where
    T: SliceMut,
{
    type ChannelMut<'a>
    where
        T::Item: 'a,
    = InterleavedChannelMut<'a, T::Item>;

    fn channel_mut(&mut self, channel: usize) -> Self::ChannelMut<'_> {
        InterleavedChannelMut::from_slice(self.value.as_mut(), channel, self.channels)
    }

    fn copy_channels(&mut self, from: usize, to: usize) {
        let frames = self.frames();

        // Safety: We're calling the copy function with internal
        // parameters which are guaranteed to be correct. `frames` is
        // guaranteed to reflect a valid subset of the buffer based on
        // frames, because it uses the trusted length of the provided
        // slice.
        unsafe {
            crate::utils::copy_channels_interleaved(
                self.value.as_mut_ptr(),
                self.channels,
                frames,
                from,
                to,
            );
        }
    }
}

impl<T> AsInterleavedMut<T::Item> for Interleaved<T>
where
    T: SliceMut,
{
    fn as_interleaved_mut(&mut self) -> &mut [T::Item] {
        self.value.as_mut()
    }

    fn as_interleaved_mut_ptr(&mut self) -> *mut T::Item {
        self.value.as_mut_ptr()
    }
}

impl<T> ReadBuf for Interleaved<&'_ [T]>
where
    T: Copy,
{
    fn remaining(&self) -> usize {
        self.frames()
    }

    fn advance(&mut self, n: usize) {
        self.value = self
            .value
            .get(n.saturating_mul(self.channels)..)
            .unwrap_or_default();
    }
}

impl<T> ReadBuf for Interleaved<&'_ mut [T]>
where
    T: Copy,
{
    fn remaining(&self) -> usize {
        self.frames()
    }

    fn advance(&mut self, n: usize) {
        let value = std::mem::take(&mut self.value);
        self.value = value
            .get_mut(n.saturating_mul(self.channels)..)
            .unwrap_or_default();
    }
}

impl<T> WriteBuf for Interleaved<&'_ mut [T]>
where
    T: Copy,
{
    fn remaining_mut(&self) -> usize {
        self.frames()
    }

    fn advance_mut(&mut self, n: usize) {
        let value = std::mem::take(&mut self.value);
        self.value = value
            .get_mut(n.saturating_mul(self.channels)..)
            .unwrap_or_default();
    }
}

impl<T> InterleavedBuf for Interleaved<&'_ mut [T]>
where
    T: Copy,
{
    fn reserve_frames(&mut self, frames: usize) {
        if frames > self.value.len() {
            panic!(
                "required number of frames {new_len} is larger than the wrapped buffer {len}",
                new_len = frames,
                len = self.value.len()
            );
        }
    }

    fn set_topology(&mut self, channels: usize, frames: usize) {
        let new_len = channels * frames;
        let len = self.value.len();

        let value = std::mem::take(&mut self.value);

        let value = match value.get_mut(..new_len) {
            Some(value) => value,
            None => {
                panic!(
                    "the topology {channels}:{frames} requires {new_len}, which is larger than the wrapped buffer {len}",
                    channels = channels,
                    frames = frames,
                    new_len = new_len,
                    len = len,
                );
            }
        };

        self.value = value;
        self.channels = channels;
    }
}
