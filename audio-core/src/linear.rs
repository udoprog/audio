//! Utilities for working with linear buffers.

use crate::{Channel, ChannelMut, Slice, SliceIndex, SliceMut};
use std::cmp;
use std::fmt;
use std::hash;
use std::iter;
use std::ops;
use std::slice;

/// The buffer of a single linear channel.
///
/// This doesn't provide direct access to the underlying buffer, but rather
/// allows us to copy data usinga  number of utility functions.
///
/// See [Buf::channel][crate::Buf::channel].
pub struct LinearChannel<T> {
    /// The underlying channel buffer.
    buf: T,
}

impl<T> LinearChannel<T> {
    /// Construct a linear channel buffer.
    ///
    /// The buffer provided as-is constitutes the frames of the channel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::LinearChannel;
    ///
    /// let buf: &[u32] = &[1, 3, 5, 7];
    /// let channel = LinearChannel::new(buf);
    ///
    /// assert_eq!(channel[1], 3);
    /// assert_eq!(channel[2], 5);
    /// ```
    pub fn new(buf: T) -> Self {
        Self { buf }
    }
}

impl<T> Channel for LinearChannel<T>
where
    T: SliceIndex,
{
    type Sample = T::Item;
    type Iter<'a>
    where
        T::Item: 'a,
    = LinearChannelIter<'a, T::Item>;

    fn frames(&self) -> usize {
        self.buf.as_ref().len()
    }

    fn iter(&self) -> Self::Iter<'_> {
        LinearChannelIter::new(self.buf.as_ref())
    }

    fn skip(self, n: usize) -> Self {
        Self {
            buf: self.buf.index(n..),
        }
    }

    fn tail(self, n: usize) -> Self {
        let start = self.buf.as_ref().len().saturating_sub(n);

        Self {
            buf: self.buf.index(start..),
        }
    }

    fn limit(self, limit: usize) -> Self {
        Self {
            buf: self.buf.index(..limit),
        }
    }

    fn chunk(self, n: usize, len: usize) -> Self {
        Self {
            buf: self.buf.index(n..n + len),
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
        Some(self.buf.as_ref())
    }
}

impl<T> ChannelMut for LinearChannel<T>
where
    T: SliceMut + SliceIndex,
{
    type IterMut<'a>
    where
        T::Item: 'a,
    = slice::IterMut<'a, T::Item>;

    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        self.buf.as_mut().iter_mut()
    }

    fn as_linear_mut(&mut self) -> Option<&mut [T::Item]> {
        Some(self.buf.as_mut())
    }
}

impl<T> ops::Index<usize> for LinearChannel<T>
where
    T: Slice,
{
    type Output = T::Item;

    fn index(&self, index: usize) -> &Self::Output {
        &self.buf.as_ref()[index]
    }
}

impl<'a, T> IntoIterator for LinearChannel<&'a [T]>
where
    T: Copy,
{
    type Item = T;
    type IntoIter = iter::Copied<slice::Iter<'a, T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.buf.iter().copied()
    }
}

impl<'a, T> IntoIterator for LinearChannel<&'a mut [T]> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.buf.iter_mut()
    }
}

impl<T> Clone for LinearChannel<T>
where
    T: Copy,
{
    fn clone(&self) -> Self {
        Self { buf: self.buf }
    }
}

impl<T> Copy for LinearChannel<T> where T: Copy {}

impl<T> cmp::PartialEq for LinearChannel<T>
where
    T: Slice,
    T::Item: cmp::PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.buf.as_ref().eq(other.buf.as_ref())
    }
}

impl<T> cmp::Eq for LinearChannel<T>
where
    T: Slice,
    T::Item: cmp::Eq,
{
}

impl<T> hash::Hash for LinearChannel<T>
where
    T: Slice,
    T::Item: hash::Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.buf.as_ref().hash(state)
    }
}

impl<T> cmp::PartialOrd for LinearChannel<T>
where
    T: Slice,
    T::Item: cmp::PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.buf.as_ref().partial_cmp(other.buf.as_ref())
    }
}

impl<T> cmp::Ord for LinearChannel<T>
where
    T: Slice,
    T::Item: cmp::Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.buf.as_ref().cmp(other.buf.as_ref())
    }
}

impl<T> fmt::Debug for LinearChannel<T>
where
    T: Slice,
    T::Item: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.buf.as_ref()).finish()
    }
}

/// An iterator over an interleaved channel.
///
/// Constructed through the [Channel::iter] of [LinearChannel].
#[repr(transparent)]
pub struct LinearChannelIter<'a, T>
where
    T: Copy,
{
    iter: slice::Iter<'a, T>,
}

impl<'a, T> LinearChannelIter<'a, T>
where
    T: Copy,
{
    #[inline]
    fn new(buf: &'a [T]) -> Self {
        Self { iter: buf.iter() }
    }
}

impl<'a, T> Iterator for LinearChannelIter<'a, T>
where
    T: Copy,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().copied()
    }
}

impl<'a, T> DoubleEndedIterator for LinearChannelIter<'a, T>
where
    T: Copy,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().copied()
    }
}

impl<'a, T> ExactSizeIterator for LinearChannelIter<'a, T>
where
    T: Copy,
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}
