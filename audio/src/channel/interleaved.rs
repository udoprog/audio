//! Utilities for working with interleaved channels.

use core::cmp;
use core::fmt;
use core::marker;
use core::mem;
use core::ptr;

use audio_core::{Channel, ChannelMut};

#[macro_use]
mod macros;

#[cfg(test)]
mod tests;

#[inline(always)]
fn size_from_ptr<T>(_: *const T) -> usize {
    mem::size_of::<T>()
}

interleaved_channel!('a, T, const, InterleavedChannel, align_iterable_ref);
interleaved_channel!('a, T, mut, InterleavedChannelMut, align_iterable_mut);

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
/// See [Buf::get][crate::Buf::get].
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
    /// ```
    /// use audio::channel::InterleavedChannel;
    ///
    /// let buf: &[u32] = &[1, 2];
    /// assert!(InterleavedChannel::from_slice(buf, 1, 4).is_none());
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use audio::channel::InterleavedChannel;
    ///
    /// let buf: &[u32] = &[1, 2, 3, 4, 5, 6, 7, 8];
    /// let channel = InterleavedChannel::from_slice(buf, 1, 2).unwrap();
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
    /// ```
    /// use audio::channel::InterleavedChannelMut;
    ///
    /// let buf: &mut [u32] = &mut [1, 2];
    /// assert!(InterleavedChannelMut::from_slice(buf, 1, 4).is_none());
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use audio::channel::InterleavedChannelMut;
    ///
    /// let buf: &mut [u32] = &mut [1, 2, 3, 4, 5, 6, 7, 8];
    /// let channel = InterleavedChannelMut::from_slice(buf, 1, 2).unwrap();
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

    /// Get the given frame mutably if it's in bound.
    pub fn get_mut(&mut self, n: usize) -> Option<&mut T> {
        if n < len!(self) {
            if mem::size_of::<T>() == 0 {
                Some(unsafe { &mut *(self.ptr.as_ptr() as *mut _) })
            } else {
                let add = n.saturating_mul(self.step);
                Some(unsafe { &mut *(self.ptr.as_ptr() as *mut T).add(add) })
            }
        } else {
            None
        }
    }

    /// Construct a mutable iterator over the channel.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
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
    type ChannelMut<'this> = InterleavedChannelMut<'this, Self::Sample>
    where
        Self: 'this;

    type IterMut<'this> = IterMut<'this, Self::Sample>
    where
        Self: 'this;

    #[inline]
    fn as_channel_mut(&mut self) -> Self::ChannelMut<'_> {
        InterleavedChannelMut {
            ptr: self.ptr,
            end: self.end,
            step: self.step,
            _marker: marker::PhantomData,
        }
    }

    #[inline]
    fn get_mut(&mut self, n: usize) -> Option<&mut T> {
        (*self).get_mut(n)
    }

    #[inline]
    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        (*self).iter_mut()
    }

    #[inline]
    fn try_as_linear_mut(&mut self) -> Option<&mut [T]> {
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
pub struct Iter<'a, T> {
    ptr: ptr::NonNull<T>,
    end: *const T,
    step: usize,
    _marker: marker::PhantomData<&'a [T]>,
}

impl<'a, T> Iter<'a, T> {
    /// Construct a new aligned iterator.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the provided pointer points to an appropriately sized buffer.
    #[inline]
    pub(crate) unsafe fn new_aligned(
        ptr: ptr::NonNull<T>,
        len: usize,
        offset: usize,
        channels: usize,
        step: usize,
    ) -> Self {
        let (ptr, end) = align_iterable_ref(ptr, len, offset, channels);

        Self {
            ptr,
            end,
            step,
            _marker: marker::PhantomData,
        }
    }
}

/// Allign an iterable reference with the given length.
///
/// # Safety
///
/// The caller is responsible for ensuring that the buffer is valid up until the
/// maximum of `len` and `offset` and that no exclusive access to the specified
/// range (by step) is already in use.
unsafe fn align_iterable_ref<T>(
    ptr: ptr::NonNull<T>,
    len: usize,
    offset: usize,
    max: usize,
) -> (ptr::NonNull<T>, *const T) {
    debug_assert!(
        offset <= max,
        "referencing channel out of bounds; offset={}, max={}",
        offset,
        max,
    );
    debug_assert!(
        len % max == 0,
        "number of channels misaligned with length; max={}, len={}",
        max,
        len,
    );
    debug_assert!(max <= len, "max out of bounds; max={}, len={}", max, len,);

    let ptr = ptr.as_ptr();

    let (ptr, end) = if mem::size_of::<T>() == 0 {
        let end = (ptr as *const u8).wrapping_add(len / max) as *const T;
        (ptr, end)
    } else {
        let ptr = ptr.add(offset);
        let end = ptr.wrapping_add(len) as *const T;
        (ptr, end)
    };

    let ptr = ptr::NonNull::new_unchecked(ptr);
    (ptr, end)
}

/// Allign a mutable reference with the given length.
///
/// # Safety
///
/// The caller is responsible for ensuring that the buffer is valid up until the
/// maximum of `len` and `offset` and that no exclusive access to the specified
/// range (by step) is already in use.
unsafe fn align_iterable_mut<T>(
    ptr: ptr::NonNull<T>,
    len: usize,
    offset: usize,
    max: usize,
) -> (ptr::NonNull<T>, *mut T) {
    debug_assert!(
        offset <= max,
        "referencing channel out of bounds; offset={}, max={}",
        offset,
        max,
    );
    debug_assert!(
        len % max == 0,
        "number of channels misaligned with length; max={}, len={}",
        max,
        len,
    );
    debug_assert!(max <= len, "max out of bounds; max={}, len={}", max, len,);

    let ptr = ptr.as_ptr();

    let (ptr, end) = if mem::size_of::<T>() == 0 {
        let end = (ptr as *mut u8).wrapping_add(len / max) as *mut T;
        (ptr, end)
    } else {
        let ptr = ptr.add(offset);
        let end = ptr.wrapping_add(len) as *mut T;
        (ptr, end)
    };

    let ptr = ptr::NonNull::new_unchecked(ptr);
    (ptr, end)
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
