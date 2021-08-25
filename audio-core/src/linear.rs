//! Utilities for working with linear buffers.

use crate::{Channel, ChannelMut, Slice};
use std::cmp;
use std::fmt;

#[macro_use]
mod macros;

slice_comparisons!({'a, T, const N: usize}, LinearRef<'a, T>, [T; N]);
slice_comparisons!({'a, T}, LinearRef<'a, T>, [T]);
slice_comparisons!({'a, T}, LinearRef<'a, T>, &[T]);
slice_comparisons!({'a, T}, LinearRef<'a, T>, Vec<T>);
slice_comparisons!({'a, T, const N: usize}, LinearMut<'a, T>, [T; N]);
slice_comparisons!({'a, T}, LinearMut<'a, T>, [T]);
slice_comparisons!({'a, T}, LinearMut<'a, T>, &[T]);
slice_comparisons!({'a, T}, LinearMut<'a, T>, Vec<T>);

/// The buffer of a single linear channel.
///
/// This doesn't provide direct access to the underlying buffer, but rather
/// allows us to copy data usinga  number of utility functions.
///
/// See [Buf::get][crate::Buf::get].
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct LinearRef<'a, T> {
    /// The underlying channel buffer.
    buf: &'a [T],
}

impl<'a, T> LinearRef<'a, T> {
    /// Construct a linear channel buffer.
    ///
    /// The buffer provided as-is constitutes the frames of the channel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::LinearRef;
    ///
    /// let buf: &[u32] = &[1, 3, 5, 7];
    /// let channel = LinearRef::new(buf);
    ///
    /// assert_eq!(channel.iter().nth(1), Some(3));
    /// assert_eq!(channel.iter().nth(2), Some(5));
    /// ```
    pub fn new(buf: &'a [T]) -> Self {
        Self { buf }
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

impl<'a, T> LinearRef<'a, T>
where
    T: Copy,
{
    /// Construct an immutable iterator over the linear channel.
    pub fn iter(&self) -> std::iter::Copied<std::slice::Iter<'_, T>> {
        self.buf.iter().copied()
    }
}

impl<'a, T> Channel for LinearRef<'a, T>
where
    T: Copy,
{
    type Sample = T;
    type Iter<'i>
    where
        T: 'i,
    = std::iter::Copied<std::slice::Iter<'i, T>>;

    fn frames(&self) -> usize {
        self.buf.len()
    }

    fn iter(&self) -> Self::Iter<'_> {
        (*self).iter()
    }

    fn skip(self, n: usize) -> Self {
        Self {
            buf: self.buf.get(n..).unwrap_or_default(),
        }
    }

    fn tail(self, n: usize) -> Self {
        let start = self.buf.len().saturating_sub(n);

        Self {
            buf: self.buf.get(start..).unwrap_or_default(),
        }
    }

    fn limit(self, limit: usize) -> Self {
        Self {
            buf: self.buf.get(..limit).unwrap_or_default(),
        }
    }

    fn chunk(self, n: usize, window: usize) -> Self {
        let n = n.saturating_mul(window);

        Self {
            buf: self
                .buf
                .get(n..n.saturating_add(window))
                .unwrap_or_default(),
        }
    }

    fn as_linear(&self) -> Option<&[T]> {
        Some(self.buf)
    }
}

impl<T> fmt::Debug for LinearRef<'_, T>
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
pub struct LinearMut<'a, T> {
    /// The underlying channel buffer.
    buf: &'a mut [T],
}

impl<'a, T> LinearMut<'a, T> {
    /// Construct a linear channel buffer.
    ///
    /// The buffer provided as-is constitutes the frames of the channel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::LinearMut;
    ///
    /// let buf: &mut [u32] = &mut [1, 3, 5, 7];
    /// let channel = LinearMut::new(buf);
    ///
    /// assert_eq!(channel.iter().nth(1), Some(3));
    /// assert_eq!(channel.iter().nth(2), Some(5));
    /// ```
    pub fn new(buf: &'a mut [T]) -> Self {
        Self { buf }
    }

    /// Construct an immutable iterator over the linear channel.
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.buf.iter_mut()
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

impl<'a, T> LinearMut<'a, T>
where
    T: Copy,
{
    /// Construct an immutable iterator over the linear channel.
    pub fn iter(&self) -> std::iter::Copied<std::slice::Iter<'_, T>> {
        self.buf.iter().copied()
    }
}

impl<'a, T> Channel for LinearMut<'a, T>
where
    T: Copy,
{
    type Sample = T;

    type Iter<'i>
    where
        T: 'i,
    = std::iter::Copied<std::slice::Iter<'i, T>>;

    fn frames(&self) -> usize {
        self.buf.len()
    }

    fn iter(&self) -> Self::Iter<'_> {
        self.buf.iter().copied()
    }

    fn skip(self, n: usize) -> Self {
        Self {
            buf: self.buf.get_mut(n..).unwrap_or_default(),
        }
    }

    fn tail(self, n: usize) -> Self {
        let start = self.buf.len().saturating_sub(n);

        Self {
            buf: self.buf.get_mut(start..).unwrap_or_default(),
        }
    }

    fn limit(self, limit: usize) -> Self {
        Self {
            buf: self.buf.get_mut(..limit).unwrap_or_default(),
        }
    }

    fn chunk(self, n: usize, window: usize) -> Self {
        let n = n.saturating_mul(window);

        Self {
            buf: self
                .buf
                .get_mut(n..)
                .and_then(|b| b.get_mut(..window))
                .unwrap_or_default(),
        }
    }

    fn as_linear(&self) -> Option<&[T]> {
        Some(self.buf)
    }
}

impl<'a, T> ChannelMut for LinearMut<'a, T>
where
    T: Copy,
{
    type IterMut<'i>
    where
        Self::Sample: 'i,
    = std::slice::IterMut<'i, T>;

    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        (*self).iter_mut()
    }

    fn as_linear_mut(&mut self) -> Option<&mut [Self::Sample]> {
        Some(self.buf)
    }
}

impl<T> fmt::Debug for LinearMut<'_, T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.buf.iter()).finish()
    }
}
