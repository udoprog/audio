use core::mem;
use core::ptr;

use audio_core::{
    Buf, BufMut, ExactSizeBuf, InterleavedBuf, InterleavedBufMut, ReadBuf, ResizableBuf, WriteBuf,
};

use crate::buf::interleaved::{Iter, IterChannelsMut};
use crate::channel::{InterleavedChannel, InterleavedChannelMut};
use crate::frame::{InterleavedFrame, InterleavedIterFrames, RawInterleaved};
use crate::slice::{Slice, SliceIndex, SliceMut};

/// A wrapper for an interleaved audio buffer.
///
/// See [wrap::interleaved][super::interleaved()].
pub struct Interleaved<T> {
    value: T,
    channels: usize,
    frames: usize,
}

impl<T> Interleaved<T>
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
    /// let buf = audio::wrap::interleaved(&[1, 2, 3, 4], 2);
    /// assert_eq!(buf.into_inner(), &[1, 2, 3, 4]);
    /// ```
    #[inline]
    pub fn into_inner(self) -> T {
        self.value
    }

    /// Construct an iterator over the interleaved wrapper.
    ///
    /// # Examples
    ///
    /// ```
    /// let buf = audio::wrap::interleaved(&[1, 2, 3, 4], 2);
    /// let mut it = buf.iter();
    ///
    /// assert_eq!(it.next().unwrap(), [1, 3]);
    /// assert_eq!(it.next().unwrap(), [2, 4]);
    /// ```
    #[inline]
    pub fn iter(&self) -> Iter<'_, T::Item> {
        unsafe { Iter::new_unchecked(self.value.as_ptr(), self.value.len(), self.channels) }
    }

    /// Access the raw sequential buffer.
    #[inline]
    fn as_raw(&self) -> RawInterleaved<T::Item>
    where
        T: Slice,
    {
        // SAFETY: construction of the current buffer ensures this is safe.
        unsafe { RawInterleaved::new(self.value.as_ref(), self.channels, self.frames) }
    }
}

impl<T> Interleaved<T>
where
    T: SliceMut,
{
    /// Construct an iterator over the interleaved wrapper.
    #[inline]
    pub fn iter_mut(&mut self) -> IterChannelsMut<'_, T::Item> {
        unsafe {
            IterChannelsMut::new_unchecked(self.value.as_mut_ptr(), self.value.len(), self.channels)
        }
    }
}

impl<T> Buf for Interleaved<T>
where
    T: Slice,
{
    type Sample = T::Item;

    type Channel<'this> = InterleavedChannel<'this, Self::Sample>
    where
        Self: 'this;

    type IterChannels<'this> = Iter<'this, Self::Sample>
    where
        Self: 'this;

    type Frame<'this> = InterleavedFrame<'this, T::Item>
    where
        Self: 'this;

    type IterFrames<'this> = InterleavedIterFrames<'this, T::Item>
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

    #[inline]
    fn channel(&self, channel: usize) -> Option<Self::Channel<'_>> {
        InterleavedChannel::from_slice(self.value.as_ref(), channel, self.channels)
    }

    #[inline]
    fn iter_channels(&self) -> Self::IterChannels<'_> {
        (*self).iter()
    }

    #[inline]
    fn frame(&self, frame: usize) -> Option<Self::Frame<'_>> {
        if frame >= self.frames {
            return None;
        }

        Some(InterleavedFrame::new(frame, self.as_raw()))
    }

    #[inline]
    fn iter_frames(&self) -> Self::IterFrames<'_> {
        InterleavedIterFrames::new(0, self.as_raw())
    }
}

impl<T> ExactSizeBuf for Interleaved<T>
where
    T: Slice,
{
    fn frames(&self) -> usize {
        self.frames
    }
}

impl<T> InterleavedBuf for Interleaved<T>
where
    T: Slice,
{
    type Sample = T::Item;

    #[inline]
    fn as_interleaved(&self) -> &[Self::Sample] {
        self.value.as_ref()
    }
}

impl<T> InterleavedBufMut for Interleaved<T>
where
    T: SliceMut,
{
    #[inline]
    fn as_interleaved_mut(&mut self) -> &mut [Self::Sample] {
        self.value.as_mut()
    }

    #[inline]
    fn as_interleaved_mut_ptr(&mut self) -> ptr::NonNull<Self::Sample> {
        self.value.as_mut_ptr()
    }

    #[inline]
    unsafe fn set_interleaved_topology(&mut self, channels: usize, frames: usize) {
        self.channels = channels;
        self.frames = frames;
    }
}

impl<T> BufMut for Interleaved<T>
where
    T: SliceMut,
{
    type ChannelMut<'this> = InterleavedChannelMut<'this, T::Item>
    where
        Self: 'this;

    type IterChannelsMut<'this> = IterChannelsMut<'this, Self::Sample>
    where
        Self: 'this;

    #[inline]
    fn channel_mut(&mut self, channel: usize) -> Option<Self::ChannelMut<'_>> {
        InterleavedChannelMut::from_slice(self.value.as_mut(), channel, self.channels)
    }

    #[inline]
    fn copy_channel(&mut self, from: usize, to: usize) {
        // Safety: We're calling the copy function with internal
        // parameters which are guaranteed to be correct. `frames` is
        // guaranteed to reflect a valid subset of the buffer based on
        // frames, because it uses the trusted length of the provided
        // slice.
        unsafe {
            crate::utils::copy_channels_interleaved(
                self.value.as_mut_ptr(),
                self.channels,
                self.frames,
                from,
                to,
            );
        }
    }

    #[inline]
    fn iter_channels_mut(&mut self) -> Self::IterChannelsMut<'_> {
        (*self).iter_mut()
    }
}

impl<T> ReadBuf for Interleaved<T>
where
    T: Default + SliceIndex,
{
    #[inline]
    fn remaining(&self) -> usize {
        self.frames
    }

    fn advance(&mut self, n: usize) {
        self.frames = self.frames.saturating_sub(n);
        let value = mem::take(&mut self.value);
        self.value = value.index_from(n.saturating_mul(self.channels));
    }
}

impl<T> WriteBuf for Interleaved<&'_ mut [T]>
where
    T: Copy,
{
    #[inline]
    fn remaining_mut(&self) -> usize {
        self.frames
    }

    fn advance_mut(&mut self, n: usize) {
        self.frames = self.frames.saturating_sub(n);

        let value = mem::take(&mut self.value);
        self.value = value
            .get_mut(n.saturating_mul(self.channels)..)
            .unwrap_or_default();
    }
}

impl<T> ResizableBuf for Interleaved<&'_ mut [T]>
where
    T: Copy,
{
    #[inline]
    fn try_reserve(&mut self, capacity: usize) -> bool {
        capacity <= self.value.len()
    }

    fn resize(&mut self, frames: usize) {
        if frames > self.value.len() {
            panic!(
                "required number of frames {new_len} is larger than the wrapped buffer {len}",
                new_len = frames,
                len = self.value.len()
            );
        }
    }

    fn resize_topology(&mut self, channels: usize, frames: usize) {
        let new_len = channels * frames;
        let len = self.value.len();

        let value = mem::take(&mut self.value);

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
        self.frames = frames;
    }
}
