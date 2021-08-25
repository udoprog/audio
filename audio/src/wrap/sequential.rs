use crate::sequential::{Iter, IterMut};
use audio_core::{Buf, BufMut, ExactSizeBuf, LinearChannel, LinearChannelMut, Slice, SliceMut};

/// A wrapper for a sequential audio buffer.
///
/// See [wrap::sequential][super::sequential()].
pub struct Sequential<T> {
    value: T,
    channels: usize,
    frames: usize,
}

impl<T> Sequential<T>
where
    T: Slice,
{
    pub(super) fn new(value: T, channels: usize) -> Self {
        assert!(
            channels != 0 && value.as_ref().len() % channels == 0,
            "slice provided {} doesn't match channel configuration {}",
            value.as_ref().len(),
            channels,
        );

        let frames = value.as_ref().len() / channels;
        Self {
            value,
            channels,
            frames,
        }
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

    /// Construct an iterator over all sequential channels.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let buf = audio::wrap::sequential(&[1, 2, 3, 4], 2);
    /// let mut it = buf.iter();
    ///
    /// assert_eq!(it.next().unwrap(), [1, 2]);
    /// assert_eq!(it.next().unwrap(), [3, 4]);
    /// ```
    pub fn iter(&self) -> Iter<'_, T::Item> {
        Iter::new(self.value.as_ref(), self.frames)
    }
}

impl<T> Sequential<T>
where
    T: SliceMut,
{
    /// Construct an iterator over all sequential channels.
    pub fn iter_mut(&mut self) -> IterMut<'_, T::Item> {
        IterMut::new(self.value.as_mut(), self.frames)
    }
}

impl<T> Buf for Sequential<T>
where
    T: Slice,
{
    type Sample = T::Item;

    type Channel<'a>
    where
        Self::Sample: 'a,
    = LinearChannel<'a, Self::Sample>;

    type Iter<'a>
    where
        Self::Sample: 'a,
    = Iter<'a, Self::Sample>;

    fn frames_hint(&self) -> Option<usize> {
        Some(self.frames)
    }

    fn channels(&self) -> usize {
        self.channels
    }

    fn get(&self, channel: usize) -> Option<Self::Channel<'_>> {
        let value = self
            .value
            .as_ref()
            .get(channel.saturating_mul(self.frames)..)?
            .get(..self.frames)
            .unwrap_or_default();
        Some(LinearChannel::new(value))
    }

    fn iter(&self) -> Self::Iter<'_> {
        (*self).iter()
    }
}

impl<T> ExactSizeBuf for Sequential<T>
where
    T: Slice,
{
    #[inline]
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
        Self::Sample: 'a,
    = LinearChannelMut<'a, Self::Sample>;

    type IterMut<'a>
    where
        Self::Sample: 'a,
    = IterMut<'a, Self::Sample>;

    fn get_mut(&mut self, channel: usize) -> Option<Self::ChannelMut<'_>> {
        let value = self
            .value
            .as_mut()
            .get_mut(channel.saturating_mul(self.frames)..)?;
        let value = value.get_mut(..self.frames).unwrap_or_default();
        Some(LinearChannelMut::new(value))
    }

    fn copy_channels(&mut self, from: usize, to: usize) {
        // Safety: We're calling the copy function with internal
        // parameters which are guaranteed to be correct. `channels` is
        // guaranteed to reflect a valid subset of the buffer based on
        // frames, because it uses the trusted length of the provided
        // slice.
        unsafe {
            crate::utils::copy_channels_sequential(
                self.value.as_mut_ptr(),
                self.channels,
                self.frames,
                from,
                to,
            );
        }
    }

    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        (*self).iter_mut()
    }
}
