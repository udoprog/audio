//! Utilities for working with linear buffers.

use crate::slice::Slice;
use audio_core::{Channel, ChannelMut};
use std::cmp;
use std::fmt;
use std::ops;
use std::slice;

#[macro_use]
mod macros;

mod iter;
pub use self::iter::{Iter, IterMut};

slice_comparisons!({'a, T, const N: usize}, LinearChannel<'a, T>, [T; N]);
slice_comparisons!({'a, T}, LinearChannel<'a, T>, [T]);
slice_comparisons!({'a, T}, LinearChannel<'a, T>, &[T]);
slice_comparisons!({'a, T}, LinearChannel<'a, T>, Vec<T>);
slice_comparisons!({'a, T, const N: usize}, LinearChannelMut<'a, T>, [T; N]);
slice_comparisons!({'a, T}, LinearChannelMut<'a, T>, [T]);
slice_comparisons!({'a, T}, LinearChannelMut<'a, T>, &[T]);
slice_comparisons!({'a, T}, LinearChannelMut<'a, T>, Vec<T>);

/// The buffer of a single linear channel.
///
/// This doesn't provide direct access to the underlying buffer, but rather
/// allows us to copy data usinga  number of utility functions.
///
/// See [Buf::get][crate::Buf::get].
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct LinearChannel<'a, T> {
    /// The underlying channel buffer.
    buf: &'a [T],
}

impl<'a, T> LinearChannel<'a, T> {
    /// Construct a linear channel buffer.
    ///
    /// The buffer provided as-is constitutes the frames of the channel.
    ///
    /// # Examples
    ///
    /// ```
    /// use audio::channel::LinearChannel;
    ///
    /// let buf: &[u32] = &[1, 3, 5, 7];
    /// let channel = LinearChannel::new(buf);
    ///
    /// assert_eq!(channel.iter().nth(1), Some(3));
    /// assert_eq!(channel.iter().nth(2), Some(5));
    /// ```
    #[inline]
    pub fn new(buf: &'a [T]) -> Self {
        Self { buf }
    }

    /// Get the given frame in the linear channel.
    #[inline]
    pub fn get(&self, n: usize) -> Option<T>
    where
        T: Copy,
    {
        self.buf.get(n).copied()
    }

    /// Construct an immutable iterator over the linear channel.
    #[inline]
    pub fn iter(&self) -> Iter<'_, T>
    where
        T: Copy,
    {
        Iter::new(self.buf)
    }

    /// Convert the channel into the underlying buffer.
    #[inline]
    pub fn into_ref(self) -> &'a [T] {
        self.buf
    }

    /// Get a reference to the underlying buffer.
    #[inline]
    pub fn as_ref(&self) -> &[T] {
        self.buf
    }
}

impl<'a, T> Channel for LinearChannel<'a, T>
where
    T: Copy,
{
    type Sample = T;

    type Channel<'this> = LinearChannel<'this, Self::Sample>
    where
        Self: 'this;

    type Iter<'this> = Iter<'this, Self::Sample>
    where
        Self: 'this;

    #[inline]
    fn as_channel(&self) -> Self::Channel<'_> {
        Self { buf: self.buf }
    }

    #[inline]
    fn len(&self) -> usize {
        self.buf.len()
    }

    #[inline]
    fn get(&self, n: usize) -> Option<Self::Sample> {
        (*self).get(n)
    }

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        (*self).iter()
    }

    #[inline]
    fn skip(self, n: usize) -> Self {
        Self {
            buf: self.buf.get(n..).unwrap_or_default(),
        }
    }

    #[inline]
    fn tail(self, n: usize) -> Self {
        let start = self.buf.len().saturating_sub(n);

        Self {
            buf: self.buf.get(start..).unwrap_or_default(),
        }
    }

    #[inline]
    fn limit(self, limit: usize) -> Self {
        Self {
            buf: self.buf.get(..limit).unwrap_or_default(),
        }
    }

    #[inline]
    fn try_as_linear(&self) -> Option<&[T]> {
        Some(self.buf)
    }
}

impl<T> fmt::Debug for LinearChannel<'_, T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.buf).finish()
    }
}

/// The buffer of a single linear channel.
///
/// This doesn't provide direct access to the underlying buffer, but rather
/// allows us to copy data usinga  number of utility functions.
///
/// See [Buf::get][crate::Buf::get].
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LinearChannelMut<'a, T> {
    /// The underlying channel buffer.
    buf: &'a mut [T],
}

impl<'a, T> LinearChannelMut<'a, T> {
    /// Construct a linear channel buffer.
    ///
    /// The buffer provided as-is constitutes the frames of the channel.
    ///
    /// # Examples
    ///
    /// ```
    /// use audio::channel::LinearChannelMut;
    ///
    /// let buf: &mut [u32] = &mut [1, 3, 5, 7];
    /// let channel = LinearChannelMut::new(buf);
    ///
    /// assert_eq!(channel.iter().nth(1), Some(3));
    /// assert_eq!(channel.iter().nth(2), Some(5));
    /// ```
    #[inline]
    pub fn new(buf: &'a mut [T]) -> Self {
        Self { buf }
    }

    /// Get the given frame.
    #[inline]
    pub fn get(&self, n: usize) -> Option<T>
    where
        T: Copy,
    {
        self.buf.get(n).copied()
    }

    /// Construct an iterator over the linear channel.
    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter::new(self.buf)
    }

    /// Get a mutable reference to the given frame.
    #[inline]
    pub fn get_mut(&mut self, n: usize) -> Option<&mut T> {
        self.buf.get_mut(n)
    }

    /// Construct an immutable iterator over the linear channel.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut::new(self.buf)
    }

    /// Convert the channel into the underlying buffer.
    #[inline]
    pub fn into_ref(self) -> &'a [T] {
        self.buf
    }

    /// Get a reference to the underlying buffer.
    #[inline]
    pub fn as_ref(&self) -> &[T] {
        self.buf
    }

    /// Convert the channel into the underlying mutable buffer.
    #[inline]
    pub fn into_mut(self) -> &'a mut [T] {
        self.buf
    }

    /// Get a mutable reference to the underlying buffer.
    #[inline]
    pub fn as_mut(&mut self) -> &mut [T] {
        self.buf
    }
}

impl<T> audio_core::LinearChannel for LinearChannel<'_, T>
where
    T: Copy,
{
    #[inline]
    fn as_linear_channel(&self) -> &[Self::Sample] {
        self.buf
    }
}

impl<T, I> ops::Index<I> for LinearChannel<'_, T>
where
    I: slice::SliceIndex<[T]>,
{
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        self.buf.index(index)
    }
}

impl<'a, T> Channel for LinearChannelMut<'a, T>
where
    T: Copy,
{
    type Sample = T;

    type Channel<'this> = LinearChannel<'this, Self::Sample>
    where
        Self: 'this;

    type Iter<'this> = Iter<'this, Self::Sample>
    where
        Self: 'this;

    #[inline]
    fn as_channel(&self) -> Self::Channel<'_> {
        LinearChannel { buf: self.buf }
    }

    #[inline]
    fn len(&self) -> usize {
        self.buf.len()
    }

    #[inline]
    fn get(&self, n: usize) -> Option<Self::Sample> {
        (*self).get(n)
    }

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        (*self).iter()
    }

    #[inline]
    fn skip(self, n: usize) -> Self {
        Self {
            buf: self.buf.get_mut(n..).unwrap_or_default(),
        }
    }

    #[inline]
    fn tail(self, n: usize) -> Self {
        let start = self.buf.len().saturating_sub(n);

        Self {
            buf: self.buf.get_mut(start..).unwrap_or_default(),
        }
    }

    #[inline]
    fn limit(self, limit: usize) -> Self {
        Self {
            buf: self.buf.get_mut(..limit).unwrap_or_default(),
        }
    }

    #[inline]
    fn try_as_linear(&self) -> Option<&[T]> {
        Some(self.buf)
    }
}

impl<'a, T> ChannelMut for LinearChannelMut<'a, T>
where
    T: Copy,
{
    type ChannelMut<'this> = LinearChannelMut<'this, T>
    where
        Self: 'this;

    type IterMut<'this> = IterMut<'this, T>
    where
        Self: 'this;

    #[inline]
    fn as_channel_mut(&mut self) -> Self::ChannelMut<'_> {
        LinearChannelMut { buf: self.buf }
    }

    #[inline]
    fn get_mut(&mut self, n: usize) -> Option<&mut Self::Sample> {
        (*self).get_mut(n)
    }

    #[inline]
    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        (*self).iter_mut()
    }

    #[inline]
    fn try_as_linear_mut(&mut self) -> Option<&mut [Self::Sample]> {
        Some(self.buf)
    }
}

impl<T> audio_core::LinearChannel for LinearChannelMut<'_, T>
where
    T: Copy,
{
    #[inline]
    fn as_linear_channel(&self) -> &[Self::Sample] {
        self.buf
    }
}

impl<T> audio_core::LinearChannelMut for LinearChannelMut<'_, T>
where
    T: Copy,
{
    #[inline]
    fn as_linear_channel_mut(&mut self) -> &mut [Self::Sample] {
        self.buf
    }
}

impl<T> fmt::Debug for LinearChannelMut<'_, T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.buf.iter()).finish()
    }
}

impl<T, I> ops::Index<I> for LinearChannelMut<'_, T>
where
    I: slice::SliceIndex<[T]>,
{
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        self.buf.index(index)
    }
}

impl<T, I> ops::IndexMut<I> for LinearChannelMut<'_, T>
where
    I: slice::SliceIndex<[T]>,
{
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.buf.index_mut(index)
    }
}
