use core::marker;
use core::ptr;

use crate::channel::{InterleavedChannel, InterleavedChannelMut};

/// An immutable iterator over an interleaved buffer.
pub struct Iter<'a, T> {
    ptr: ptr::NonNull<T>,
    len: usize,
    channel: usize,
    channels: usize,
    _marker: marker::PhantomData<&'a [T]>,
}

impl<'a, T> Iter<'a, T> {
    /// Construct a new unchecked iterator.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the pointed to buffer is a valid immutable
    /// interleaved region of data.
    pub(crate) unsafe fn new_unchecked(ptr: ptr::NonNull<T>, len: usize, channels: usize) -> Self {
        Self {
            ptr,
            len,
            channel: 0,
            channels,
            _marker: marker::PhantomData,
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: Copy,
{
    type Item = InterleavedChannel<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.channel == self.channels {
            return None;
        }

        let channel = self.channel;
        self.channel += 1;

        unsafe {
            Some(InterleavedChannel::new_unchecked(
                self.ptr,
                self.len,
                channel,
                self.channels,
            ))
        }
    }
}

/// An mutable iterator over an interleaved buffer.
pub struct IterChannelsMut<'a, T> {
    ptr: ptr::NonNull<T>,
    len: usize,
    channel: usize,
    channels: usize,
    _marker: marker::PhantomData<&'a mut [T]>,
}

impl<'a, T> IterChannelsMut<'a, T> {
    /// Construct a new unchecked iterator.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the pointed to buffer is a valid mutable
    /// interleaved region of data.
    pub(crate) unsafe fn new_unchecked(ptr: ptr::NonNull<T>, len: usize, channels: usize) -> Self {
        Self {
            ptr,
            len,
            channel: 0,
            channels,
            _marker: marker::PhantomData,
        }
    }
}

impl<'a, T> Iterator for IterChannelsMut<'a, T>
where
    T: Copy,
{
    type Item = InterleavedChannelMut<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.channel == self.channels {
            return None;
        }

        let channel = self.channel;
        self.channel += 1;

        unsafe {
            Some(InterleavedChannelMut::new_unchecked(
                self.ptr,
                self.len,
                channel,
                self.channels,
            ))
        }
    }
}
