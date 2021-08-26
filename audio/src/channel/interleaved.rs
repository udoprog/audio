//! Utilities for working with interleaved channels.

use core::{Channel, ChannelMut};
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

interleaved_channel!('a, T, const, InterleavedRef);
interleaved_channel!('a, T, mut, InterleavedMut);

comparisons!({'a, T}, InterleavedRef<'a, T>, InterleavedMut<'a, T>);
comparisons!({'a, T}, InterleavedMut<'a, T>, InterleavedRef<'a, T>);

slice_comparisons!({'a, T, const N: usize}, InterleavedRef<'a, T>, [T; N]);
slice_comparisons!({'a, T}, InterleavedRef<'a, T>, [T]);
slice_comparisons!({'a, T}, InterleavedRef<'a, T>, &[T]);
slice_comparisons!({'a, T}, InterleavedRef<'a, T>, Vec<T>);
slice_comparisons!({'a, T, const N: usize}, InterleavedMut<'a, T>, [T; N]);
slice_comparisons!({'a, T}, InterleavedMut<'a, T>, [T]);
slice_comparisons!({'a, T}, InterleavedMut<'a, T>, &[T]);
slice_comparisons!({'a, T}, InterleavedMut<'a, T>, Vec<T>);

/// The buffer of a single interleaved channel.
///
/// This doesn't provide direct access to the underlying buffer, but rather
/// allows us to copy data usinga  number of utility functions.
///
/// See [Buf::get][crate::Buf::get].
pub struct InterleavedRef<'a, T> {
    /// The base pointer of the buffer.
    ptr: ptr::NonNull<T>,
    /// The end pointer of the buffer.
    end: *const T,
    /// The number of channels in the interleaved buffer.
    step: usize,
    /// The market indicating the kind of the channel.
    _marker: marker::PhantomData<&'a [T]>,
}

impl<'a, T> InterleavedRef<'a, T> {
    /// Construct an interleaved channel buffer from a slice.
    ///
    /// This is a safe function since the data being referenced is both bounds
    /// checked and is associated with the lifetime of the structure.
    ///
    /// Returns `None` if the channel configuration is not valid. That is either
    /// true if the given number of `channels` cannot fit within it or if the
    /// selected `channel` does not fit within the specified `channels`.
    ///
    /// ```
    /// use audio::channel::InterleavedRef;
    ///
    /// let buf: &[u32] = &[1, 2];
    /// assert!(InterleavedRef::from_slice(buf, 1, 4).is_none());
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use audio::channel::InterleavedRef;
    ///
    /// let buf: &[u32] = &[1, 2, 3, 4, 5, 6, 7, 8];
    /// let channel = InterleavedRef::from_slice(buf, 1, 2).unwrap();
    ///
    /// assert_eq!(channel.get(1), Some(4));
    /// assert_eq!(channel.get(2), Some(6));
    /// ```
    pub fn from_slice(data: &'a [T], channel: usize, channels: usize) -> Option<Self> {
        if data.len().checked_rem(channels)? != 0 || channel >= channels {
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
/// See [BufMut::get_mut][crate::BufMut::get_mut].
pub struct InterleavedMut<'a, T> {
    /// The base pointer of the buffer.
    ptr: ptr::NonNull<T>,
    /// The size of the buffer.
    end: *mut T,
    /// The number of channels in the interleaved buffer.
    step: usize,
    /// The market indicating the kind of the channel.
    _marker: marker::PhantomData<&'a mut [T]>,
}

impl<'a, T> InterleavedMut<'a, T> {
    /// Construct an interleaved channel buffer from a slice.
    ///
    /// This is a safe function since the data being referenced is both bounds
    /// checked and is associated with the lifetime of the structure.
    ///
    /// Returns `None` if the channel configuration is not valid. That is either true if
    /// the given number of `channels` cannot fit within it or if the selected
    /// `channel` does not fit within the specified `channels`.
    ///
    /// ```
    /// use audio::channel::InterleavedMut;
    ///
    /// let buf: &mut [u32] = &mut [1, 2];
    /// assert!(InterleavedMut::from_slice(buf, 1, 4).is_none());
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use audio::channel::InterleavedMut;
    ///
    /// let buf: &mut [u32] = &mut [1, 2, 3, 4, 5, 6, 7, 8];
    /// let channel = InterleavedMut::from_slice(buf, 1, 2).unwrap();
    ///
    /// assert_eq!(channel.get(1), Some(4));
    /// assert_eq!(channel.get(2), Some(6));
    /// ```
    pub fn from_slice(data: &'a mut [T], channel: usize, channels: usize) -> Option<Self> {
        if data.len().checked_rem(channels)? != 0 || channel >= channels {
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
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            ptr: self.ptr,
            end: self.end,
            step: self.step,
            _marker: marker::PhantomData,
        }
    }
}

impl<'a, T> ChannelMut for InterleavedMut<'a, T>
where
    T: Copy,
{
    type IterMut<'s>
    where
        T: 's,
    = IterMut<'s, T>;

    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        (*self).iter_mut()
    }

    fn as_linear_mut(&mut self) -> Option<&mut [T]> {
        None
    }
}

impl<T> Clone for InterleavedRef<'_, T> {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            end: self.end,
            step: self.step,
            _marker: marker::PhantomData,
        }
    }
}

/// Note: InterleavedRef is always Copy since it represents an immutable
/// buffer.
impl<T> Copy for InterleavedRef<'_, T> {}

/// An immutable iterator.
pub struct Iter<'a, T> {
    ptr: ptr::NonNull<T>,
    end: *const T,
    step: usize,
    _marker: marker::PhantomData<&'a [T]>,
}

/// A mutable iterator.
pub struct IterMut<'a, T> {
    ptr: ptr::NonNull<T>,
    end: *mut T,
    step: usize,
    _marker: marker::PhantomData<&'a mut [T]>,
}

iterator!(struct Iter -> *const T, T, const, {/* no mut */});
iterator!(struct IterMut -> *mut T, &'a mut T, mut, {&mut});
