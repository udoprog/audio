//! Utilities for working with interleaved channels.

use crate::{Channel, ChannelMut, SliceIndex, SliceMut};
use std::cmp;
use std::fmt;
use std::hash;
use std::iter;
use std::ops;
use std::slice;

/// The buffer of a single interleaved channel.
///
/// This doesn't provide direct access to the underlying buffer, but rather
/// allows us to copy data usinga  number of utility functions.
///
/// See [Buf::channel][crate::Buf::channel].
pub struct InterleavedChannel<T> {
    buf: T,
    /// The number of channels in the interleaved buffer.
    channels: usize,
    /// The channel that is being accessed.
    channel: usize,
}

impl<T> InterleavedChannel<T> {
    /// Construct an interleaved channel buffer.
    ///
    /// The provided buffer must be the complete buffer, which includes *all*
    /// other channels. The provided `channels` argument is the total number of
    /// channels in this buffer, and `channel` indicates which specific channel
    /// this buffer belongs to.
    ///
    /// Note that this is typically not used directly, but instead through an
    /// abstraction which makes sure to provide the correct parameters.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::InterleavedChannel;
    ///
    /// let buf: &[u32] = &[1, 2, 3, 4, 5, 6, 7, 8];
    /// let channel = InterleavedChannel::new(buf, 2, 1);
    ///
    /// assert_eq!(channel[1], 4);
    /// assert_eq!(channel[2], 6);
    /// ```
    pub fn new(buf: T, channels: usize, channel: usize) -> Self {
        Self {
            buf,
            channels,
            channel,
        }
    }
}

impl<T> Channel for InterleavedChannel<T>
where
    T: SliceIndex,
{
    type Sample = T::Item;

    type Iter<'a>
    where
        T::Item: 'a,
    = InterleavedChannelIter<'a, T::Item>;

    fn frames(&self) -> usize {
        self.buf.as_ref().len() / self.channels
    }

    fn iter(&self) -> Self::Iter<'_> {
        InterleavedChannelIter::new(self.channel, self.channels, self.buf.as_ref())
    }

    fn skip(self, n: usize) -> Self {
        Self {
            buf: self.buf.index(n * self.channels..),
            channels: self.channels,
            channel: self.channel,
        }
    }

    fn tail(self, n: usize) -> Self {
        let start = self.buf.as_ref().len().saturating_sub(n * self.channels);

        Self {
            buf: self.buf.index(start..),
            channels: self.channels,
            channel: self.channel,
        }
    }

    fn limit(self, limit: usize) -> Self {
        Self {
            buf: self.buf.index(..limit * self.channels),
            channels: self.channels,
            channel: self.channel,
        }
    }

    fn chunk(self, n: usize, len: usize) -> Self {
        let len = len * self.channels;
        let n = n * len;

        Self {
            buf: self.buf.index(n..n + len),
            channels: self.channels,
            channel: self.channel,
        }
    }

    fn chunks(&self, chunk: usize) -> usize {
        let len = self.frames();

        if len % chunk == 0 {
            len / chunk
        } else {
            len / chunk + 1
        }
    }

    fn as_linear(&self) -> Option<&[T::Item]> {
        None
    }
}

impl<T> ChannelMut for InterleavedChannel<T>
where
    T: SliceMut + SliceIndex,
{
    type IterMut<'a>
    where
        T::Item: 'a,
    = iter::StepBy<slice::IterMut<'a, T::Item>>;

    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        let start = usize::min(self.channel, self.buf.as_ref().len());

        self.buf
            .as_mut()
            .get_mut(start..)
            .unwrap_or_default()
            .iter_mut()
            .step_by(self.channels)
    }

    fn as_linear_mut(&mut self) -> Option<&mut [T::Item]> {
        None
    }
}

impl<'a, T> IntoIterator for InterleavedChannel<&'a [T]>
where
    T: Copy,
{
    type Item = T;
    type IntoIter = InterleavedChannelIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        InterleavedChannelIter::new(self.channel, self.channels, self.buf.as_ref())
    }
}

impl<'a, T> IntoIterator for InterleavedChannel<&'a mut [T]> {
    type Item = &'a mut T;
    type IntoIter = iter::StepBy<slice::IterMut<'a, T>>;

    fn into_iter(self) -> Self::IntoIter {
        let start = usize::min(self.channel, self.buf.len());

        self.buf
            .get_mut(start..)
            .unwrap_or_default()
            .iter_mut()
            .step_by(self.channels)
    }
}

impl<T> ops::Index<usize> for InterleavedChannel<T>
where
    T: SliceIndex,
{
    type Output = T::Item;

    fn index(&self, index: usize) -> &Self::Output {
        &self.buf.as_ref()[self.channel + self.channels * index]
    }
}

impl<T> cmp::PartialEq for InterleavedChannel<T>
where
    T: SliceIndex,
    T::Item: cmp::PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<T> cmp::Eq for InterleavedChannel<T>
where
    T: SliceIndex,
    T::Item: cmp::Eq,
{
}

impl<T> hash::Hash for InterleavedChannel<T>
where
    T: SliceIndex,
    T::Item: hash::Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        for item in self.iter() {
            item.hash(state);
        }
    }
}

impl<T> cmp::PartialOrd for InterleavedChannel<T>
where
    T: SliceIndex,
    T::Item: cmp::PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.iter().partial_cmp(other.iter())
    }
}

impl<T> cmp::Ord for InterleavedChannel<T>
where
    T: SliceIndex,
    T::Item: cmp::Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.iter().cmp(other.iter())
    }
}

impl<T> fmt::Debug for InterleavedChannel<T>
where
    T: SliceIndex,
    T::Item: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

/// An iterator over an interleaved channel.
///
/// Constructed through the [Channel::iter] of [InterleavedChannel].
#[repr(transparent)]
pub struct InterleavedChannelIter<'a, T>
where
    T: Copy,
{
    iter: iter::StepBy<slice::Iter<'a, T>>,
}

impl<'a, T> InterleavedChannelIter<'a, T>
where
    T: Copy,
{
    #[inline]
    fn new(channel: usize, channels: usize, buf: &'a [T]) -> Self {
        let start = usize::min(channel, buf.len());

        let iter = buf
            .get(start..)
            .unwrap_or_default()
            .iter()
            .step_by(channels);

        Self { iter }
    }
}

impl<'a, T> Iterator for InterleavedChannelIter<'a, T>
where
    T: Copy,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().copied()
    }
}

impl<'a, T> DoubleEndedIterator for InterleavedChannelIter<'a, T>
where
    T: Copy,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().copied()
    }
}

impl<'a, T> ExactSizeIterator for InterleavedChannelIter<'a, T>
where
    T: Copy,
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}
