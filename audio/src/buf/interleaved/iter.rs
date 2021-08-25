use core::{InterleavedMut, InterleavedRef};
use std::marker;
use std::ptr;

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
    type Item = InterleavedRef<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.channel == self.channels {
            return None;
        }

        let channel = self.channel;
        self.channel += 1;

        unsafe {
            Some(InterleavedRef::new_unchecked(
                self.ptr,
                self.len,
                channel,
                self.channels,
            ))
        }
    }
}

/// An mutable iterator over an interleaved buffer.
pub struct IterMut<'a, T> {
    ptr: ptr::NonNull<T>,
    len: usize,
    channel: usize,
    channels: usize,
    _marker: marker::PhantomData<&'a mut [T]>,
}

impl<'a, T> IterMut<'a, T> {
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

impl<'a, T> Iterator for IterMut<'a, T>
where
    T: Copy,
{
    type Item = InterleavedMut<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.channel == self.channels {
            return None;
        }

        let channel = self.channel;
        self.channel += 1;

        unsafe {
            Some(InterleavedMut::new_unchecked(
                self.ptr,
                self.len,
                channel,
                self.channels,
            ))
        }
    }
}