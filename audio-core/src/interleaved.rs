//! Utilities for working with interleaved channels.

use crate::{Channel, ChannelMut};
use std::cmp;
use std::fmt;
use std::marker;
use std::mem;
use std::ptr;

#[macro_use]
mod macros;

#[cfg(test)]
mod tests;

#[inline(always)]
fn size_from_ptr<T>(_: *const T) -> usize {
    mem::size_of::<T>()
}

interleaved_channel!('a, T, const, InterleavedChannel);
interleaved_channel!('a, T, mut, InterleavedChannelMut);

comparisons!({'a, T}, InterleavedChannel<'a, T>, InterleavedChannelMut<'a, T>);
comparisons!({'a, T}, InterleavedChannelMut<'a, T>, InterleavedChannel<'a, T>);

/// The buffer of a single interleaved channel.
///
/// This doesn't provide direct access to the underlying buffer, but rather
/// allows us to copy data usinga  number of utility functions.
///
/// See [Buf::channel][crate::Buf::channel].
pub struct InterleavedChannel<'a, T> {
    /// The base pointer of the buffer.
    ptr: ptr::NonNull<T>,
    /// The end pointer of the buffer.
    end: *const T,
    /// The number of channels in the interleaved buffer.
    step: usize,
    /// The market indicating the kind of the channel.
    _marker: marker::PhantomData<&'a [T]>,
}

impl<'a, T> InterleavedChannel<'a, T> {
    /// Construct an interleaved channel buffer from a slice.
    ///
    /// This is a safe function since the data being referenced is both bounds
    /// checked and is associated with the lifetime of the structure.
    ///
    /// # Panics
    ///
    /// Panics if the channel configuration is not valid. That is either true if
    /// the given number of `channels` cannot fit within it or if the selected
    /// `channel` does not fit within the specified `channels`.
    ///
    /// ```rust,should_panic
    /// use audio::InterleavedChannel;
    ///
    /// let buf: &[u32] = &[1, 2];
    /// InterleavedChannel::from_slice(buf, 1, 4);
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::{Channel, InterleavedChannel};
    ///
    /// let buf: &[u32] = &[1, 2, 3, 4, 5, 6, 7, 8];
    /// let channel = InterleavedChannel::from_slice(buf, 1, 2);
    ///
    /// assert_eq!(channel.iter().nth(1), Some(4));
    /// assert_eq!(channel.iter().nth(2), Some(6));
    /// ```
    pub fn from_slice(data: &'a [T], channel: usize, channels: usize) -> Self {
        assert!(channels <= data.len());
        assert!(channel < channels);

        unsafe {
            Self::new_unchecked(
                ptr::NonNull::new_unchecked(data.as_ptr() as *mut _),
                data.len(),
                channel,
                channels,
            )
        }
    }
}

/// The buffer of a single interleaved channel.
///
/// This doesn't provide direct access to the underlying buffer, but rather
/// allows us to copy data usinga  number of utility functions.
///
/// See [Buf::channel][crate::Buf::channel].
pub struct InterleavedChannelMut<'a, T> {
    /// The base pointer of the buffer.
    ptr: ptr::NonNull<T>,
    /// The size of the buffer.
    end: *mut T,
    /// The number of channels in the interleaved buffer.
    step: usize,
    /// The market indicating the kind of the channel.
    _marker: marker::PhantomData<&'a mut [T]>,
}

impl<'a, T> InterleavedChannelMut<'a, T> {
    /// Construct an interleaved channel buffer from a slice.
    ///
    /// This is a safe function since the data being referenced is both bounds
    /// checked and is associated with the lifetime of the structure.
    ///
    /// # Panics
    ///
    /// Panics if the channel configuration is not valid. That is either true if
    /// the given number of `channels` cannot fit within it or if the selected
    /// `channel` does not fit within the specified `channels`.
    ///
    /// ```rust,should_panic
    /// use audio::InterleavedChannelMut;
    ///
    /// let buf: &mut [u32] = &mut [1, 2];
    /// InterleavedChannelMut::from_slice(buf, 1, 4);
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::{Channel, InterleavedChannelMut};
    ///
    /// let buf: &mut [u32] = &mut [1, 2, 3, 4, 5, 6, 7, 8];
    /// let channel = InterleavedChannelMut::from_slice(buf, 1, 2);
    ///
    /// assert_eq!(channel.iter().nth(1), Some(4));
    /// assert_eq!(channel.iter().nth(2), Some(6));
    /// ```
    pub fn from_slice(data: &'a mut [T], channel: usize, channels: usize) -> Self {
        assert!(channels <= data.len());
        assert!(channel < channels);

        unsafe {
            Self::new_unchecked(
                ptr::NonNull::new_unchecked(data.as_mut_ptr()),
                data.len(),
                channel,
                channels,
            )
        }
    }
}

impl<'a, T> ChannelMut for InterleavedChannelMut<'a, T>
where
    T: Copy,
{
    type IterMut<'s>
    where
        T: 's,
    = InterleavedChannelMutIter<'s, T>;

    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        InterleavedChannelMutIter {
            ptr: self.ptr,
            end: self.end,
            step: self.step,
            _marker: marker::PhantomData,
        }
    }

    fn as_linear_mut(&mut self) -> Option<&mut [T]> {
        None
    }
}

impl<T> Clone for InterleavedChannel<'_, T> {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            end: self.end,
            step: self.step,
            _marker: marker::PhantomData,
        }
    }
}

/// Note: InterleavedChannel is always Copy since it represents an immutable
/// buffer.
impl<T> Copy for InterleavedChannel<'_, T> {}

/// An immutable iterator.
pub struct InterleavedChannelIter<'a, T> {
    ptr: ptr::NonNull<T>,
    end: *const T,
    step: usize,
    _marker: marker::PhantomData<&'a [T]>,
}

/// A mutable iterator.
pub struct InterleavedChannelMutIter<'a, T> {
    ptr: ptr::NonNull<T>,
    end: *mut T,
    step: usize,
    _marker: marker::PhantomData<&'a mut [T]>,
}

iterator!(struct InterleavedChannelIter -> *const T, T, const, {/* no mut */});
iterator!(struct InterleavedChannelMutIter -> *mut T, &'a mut T, mut, {&mut});
