use crate::buf::interleaved::{Iter, IterMut};
use core::{
    AsInterleaved, AsInterleavedMut, Buf, BufMut, ExactSizeBuf, InterleavedMut, InterleavedRef,
    ReadBuf, Slice, SliceIndex, SliceMut, WriteBuf,
};
use std::ptr;

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
    /// ```rust
    /// let buffer = audio::wrap::interleaved(&[1, 2, 3, 4], 2);
    /// assert_eq!(buffer.into_inner(), &[1, 2, 3, 4]);
    /// ```
    pub fn into_inner(self) -> T {
        self.value
    }

    /// Construct an iterator over the interleaved wrapper.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let buf = audio::wrap::interleaved(&[1, 2, 3, 4], 2);
    /// let mut it = buf.iter();
    ///
    /// assert_eq!(it.next().unwrap(), [1, 3]);
    /// assert_eq!(it.next().unwrap(), [2, 4]);
    /// ```
    pub fn iter(&self) -> Iter<'_, T::Item> {
        unsafe { Iter::new_unchecked(self.value.as_ptr(), self.value.len(), self.channels) }
    }
}

impl<T> Interleaved<T>
where
    T: SliceMut,
{
    /// Construct an iterator over the interleaved wrapper.
    pub fn iter_mut(&mut self) -> IterMut<'_, T::Item> {
        unsafe { IterMut::new_unchecked(self.value.as_mut_ptr(), self.value.len(), self.channels) }
    }
}

impl<T> Buf for Interleaved<T>
where
    T: Slice,
{
    type Sample = T::Item;

    type Channel<'a>
    where
        Self::Sample: 'a,
    = InterleavedRef<'a, Self::Sample>;

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
        InterleavedRef::from_slice(self.value.as_ref(), channel, self.channels)
    }

    fn iter(&self) -> Self::Iter<'_> {
        (*self).iter()
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
    = InterleavedMut<'a, T::Item>;

    type IterMut<'a>
    where
        Self::Sample: 'a,
    = IterMut<'a, Self::Sample>;

    fn get_mut(&mut self, channel: usize) -> Option<Self::ChannelMut<'_>> {
        InterleavedMut::from_slice(self.value.as_mut(), channel, self.channels)
    }

    fn copy_channels(&mut self, from: usize, to: usize) {
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

    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        (*self).iter_mut()
    }
}

impl<T> AsInterleavedMut<T::Item> for Interleaved<T>
where
    T: SliceMut,
{
    fn as_interleaved_mut(&mut self) -> &mut [T::Item] {
        self.value.as_mut()
    }

    fn as_interleaved_mut_ptr(&mut self) -> ptr::NonNull<T::Item> {
        self.value.as_mut_ptr()
    }
}

impl<T> ReadBuf for Interleaved<T>
where
    T: Default + SliceIndex,
{
    fn remaining(&self) -> usize {
        self.frames
    }

    fn advance(&mut self, n: usize) {
        self.frames = self.frames.saturating_sub(n);
        let value = std::mem::take(&mut self.value);
        self.value = value.index_from(n.saturating_mul(self.channels));
    }
}

impl<T> WriteBuf for Interleaved<&'_ mut [T]>
where
    T: Copy,
{
    fn remaining_mut(&self) -> usize {
        self.frames
    }

    fn advance_mut(&mut self, n: usize) {
        self.frames = self.frames.saturating_sub(n);

        let value = std::mem::take(&mut self.value);
        self.value = value
            .get_mut(n.saturating_mul(self.channels)..)
            .unwrap_or_default();
    }
}

impl<T> core::Interleaved for Interleaved<&'_ mut [T]>
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
        self.frames = frames;
    }
}
