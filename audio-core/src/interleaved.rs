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

slice_comparisons!({'a, T, const N: usize}, InterleavedChannel<'a, T>, [T; N]);
slice_comparisons!({'a, T}, InterleavedChannel<'a, T>, [T]);
slice_comparisons!({'a, T}, InterleavedChannel<'a, T>, &[T]);
slice_comparisons!({'a, T}, InterleavedChannel<'a, T>, Vec<T>);
slice_comparisons!({'a, T, const N: usize}, InterleavedChannelMut<'a, T>, [T; N]);
slice_comparisons!({'a, T}, InterleavedChannelMut<'a, T>, [T]);
slice_comparisons!({'a, T}, InterleavedChannelMut<'a, T>, &[T]);
slice_comparisons!({'a, T}, InterleavedChannelMut<'a, T>, Vec<T>);

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
    /// Returns `None` if the channel configuration is not valid. That is either
    /// true if the given number of `channels` cannot fit within it or if the
    /// selected `channel` does not fit within the specified `channels`.
    ///
    /// ```rust
    /// use audio::InterleavedChannel;
    ///
    /// let buf: &[u32] = &[1, 2];
    /// assert!(InterleavedChannel::from_slice(buf, 1, 4).is_none());
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::{Channel, InterleavedChannel};
    ///
    /// let buf: &[u32] = &[1, 2, 3, 4, 5, 6, 7, 8];
    /// let channel = InterleavedChannel::from_slice(buf, 1, 2).unwrap();
    ///
    /// assert_eq!(channel.iter().nth(1), Some(4));
    /// assert_eq!(channel.iter().nth(2), Some(6));
    /// ```
    pub fn from_slice(data: &'a [T], channel: usize, channels: usize) -> Option<Self> {
        if channels == 0 || data.len() % channels != 0 || channel >= channels {
            return None;
        }

        Some(unsafe {
            Self::new_unchecked(
                ptr::NonNull::new_unchecked(data.as_ptr() as *mut _),
                data.len(),
                channel,
                channels,
            )
        })
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
    /// Returns `None` if the channel configuration is not valid. That is either true if
    /// the given number of `channels` cannot fit within it or if the selected
    /// `channel` does not fit within the specified `channels`.
    ///
    /// ```rust
    /// use audio::InterleavedChannelMut;
    ///
    /// let buf: &mut [u32] = &mut [1, 2];
    /// assert!(InterleavedChannelMut::from_slice(buf, 1, 4).is_none());
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::{Channel, InterleavedChannelMut};
    ///
    /// let buf: &mut [u32] = &mut [1, 2, 3, 4, 5, 6, 7, 8];
    /// let channel = InterleavedChannelMut::from_slice(buf, 1, 2).unwrap();
    ///
    /// assert_eq!(channel.get(1), Some(4));
    /// assert_eq!(channel.get(2), Some(6));
    /// ```
    pub fn from_slice(data: &'a mut [T], channel: usize, channels: usize) -> Option<Self> {
        if channels == 0 || data.len() % channels != 0 || channel >= channels {
            return None;
        }

        Some(unsafe {
            Self::new_unchecked(
                ptr::NonNull::new_unchecked(data.as_mut_ptr()),
                data.len(),
                channel,
                channels,
            )
        })
    }

    /// Get the given frame if it's in bound.
    pub fn into_mut(self, frame: usize) -> Option<&'a mut T> {
        if frame < len!(self) {
            if mem::size_of::<T>() == 0 {
                Some(unsafe { &mut *(self.ptr.as_ptr() as *mut _) })
            } else {
                let add = frame.saturating_mul(self.step);
                Some(unsafe { &mut *(self.ptr.as_ptr() as *mut T).add(add) })
            }
        } else {
            None
        }
    }

    /// Get the given frame if it's in bound.
    pub fn get_mut(&mut self, frame: usize) -> Option<&mut T> {
        if frame < len!(self) {
            if mem::size_of::<T>() == 0 {
                Some(unsafe { &mut *(self.ptr.as_ptr() as *mut _) })
            } else {
                let add = frame.saturating_mul(self.step);
                Some(unsafe { &mut *(self.ptr.as_ptr() as *mut T).add(add) })
            }
        } else {
            None
        }
    }

    /// Construct a mutable iterator over the channel.
    pub fn iter_mut(&mut self) -> InterleavedChannelMutIter<'_, T> {
        InterleavedChannelMutIter {
            ptr: self.ptr,
            end: self.end,
            step: self.step,
            _marker: marker::PhantomData,
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
        (*self).iter_mut()
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
