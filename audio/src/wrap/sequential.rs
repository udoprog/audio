use audio_core::{Buf, BufMut, ExactSizeBuf, UniformBuf};

use crate::buf::sequential::{Iter, IterMut};
use crate::channel::{LinearChannel, LinearChannelMut};
use crate::frame::{RawSequential, SequentialFrame, SequentialFramesIter};
use crate::slice::{Slice, SliceMut};

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
    /// ```
    /// let buf = audio::wrap::sequential(&[1, 2, 3, 4], 2);
    /// assert_eq!(buf.into_inner(), &[1, 2, 3, 4]);
    /// ```
    #[inline]
    pub fn into_inner(self) -> T {
        self.value
    }

    /// Construct an iterator over all sequential channels.
    ///
    /// # Examples
    ///
    /// ```
    /// let buf = audio::wrap::sequential(&[1, 2, 3, 4], 2);
    /// let mut it = buf.iter();
    ///
    /// assert_eq!(it.next().unwrap(), [1, 2]);
    /// assert_eq!(it.next().unwrap(), [3, 4]);
    /// ```
    #[inline]
    pub fn iter(&self) -> Iter<'_, T::Item> {
        Iter::new(self.value.as_ref(), self.frames)
    }

    /// Access the raw sequential buffer.
    #[inline]
    fn as_raw(&self) -> RawSequential<T::Item>
    where
        T: Slice,
    {
        // SAFETY: construction of the current buffer ensures this is safe.
        unsafe { RawSequential::new(self.value.as_ref(), self.channels, self.frames) }
    }
}

impl<T> Sequential<T>
where
    T: SliceMut,
{
    /// Construct an iterator over all sequential channels.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, T::Item> {
        IterMut::new(self.value.as_mut(), self.frames)
    }
}

impl<T> Buf for Sequential<T>
where
    T: Slice,
{
    type Sample = T::Item;

    type Channel<'this> = LinearChannel<'this, Self::Sample>
    where
        Self: 'this;

    type Iter<'this> = Iter<'this, Self::Sample>
    where
        Self: 'this;

    #[inline]
    fn frames_hint(&self) -> Option<usize> {
        Some(self.frames)
    }

    #[inline]
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

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        (*self).iter()
    }
}

impl<T> UniformBuf for Sequential<T>
where
    T: Slice,
{
    type Frame<'this> = SequentialFrame<'this, T::Item>
    where
        Self: 'this;

    type FramesIter<'this> = SequentialFramesIter<'this, T::Item>
    where
        Self: 'this;

    #[inline]
    fn get_frame(&self, frame: usize) -> Option<Self::Frame<'_>> {
        if frame >= self.frames {
            return None;
        }

        Some(SequentialFrame::new(frame, self.as_raw()))
    }

    #[inline]
    fn iter_frames(&self) -> Self::FramesIter<'_> {
        SequentialFramesIter::new(0, self.as_raw())
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
    type ChannelMut<'a> = LinearChannelMut<'a, Self::Sample>
    where
        Self: 'a;

    type IterMut<'a> = IterMut<'a, Self::Sample>
    where
        Self: 'a;

    fn get_mut(&mut self, channel: usize) -> Option<Self::ChannelMut<'_>> {
        let value = self
            .value
            .as_mut()
            .get_mut(channel.saturating_mul(self.frames)..)?;
        let value = value.get_mut(..self.frames).unwrap_or_default();
        Some(LinearChannelMut::new(value))
    }

    fn copy_channel(&mut self, from: usize, to: usize) {
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
