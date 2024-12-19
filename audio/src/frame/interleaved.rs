use core::marker;
use core::ptr;
use core::slice;

use audio_core::Frame;

use crate::channel::linear::Iter;

/// An unsafe wrapper around a raw sequential buffer.
pub(crate) struct RawInterleaved<T> {
    ptr: ptr::NonNull<T>,
    len: usize,
    channels: usize,
    frames: usize,
}

impl<T> Clone for RawInterleaved<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for RawInterleaved<T> {}

impl<T> RawInterleaved<T> {
    /// Construct a new raw sequential buffer.
    ///
    /// # Safety
    ///
    /// Caller must ensure that the provided buffer is within bounds of the
    /// given specification.
    pub(crate) unsafe fn new(data: &[T], channels: usize, frames: usize) -> Self {
        let len = data.len();

        debug_assert!(
            len % channels == 0,
            "data provided is not aligned with the number of channels; len={}, frames={}",
            len,
            channels,
        );

        debug_assert!(
            len % frames == 0,
            "data provided is not aligned with the number of frames; len={}, frames={}",
            len,
            frames,
        );

        Self {
            ptr: ptr::NonNull::new_unchecked(data.as_ptr() as *mut T),
            len,
            channels,
            frames,
        }
    }

    /// Access number of frames in the buffer.
    #[inline]
    pub(crate) fn frames(&self) -> usize {
        self.frames
    }

    /// Get the given sample inside of the specified frame.
    ///
    /// This checks that the given channel `n` is in bounds.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the specified `frame` is in bounds.
    pub(crate) unsafe fn get_sample(&self, frame: usize, channel: usize) -> Option<T>
    where
        T: Copy,
    {
        let index = channel.checked_add(self.channels.checked_mul(frame)?)?;

        debug_assert!(
            index < self.len,
            "index `{index}` out-of-bounds",
            index = index
        );

        if index >= self.len {
            return None;
        }

        Some(*self.ptr.as_ptr().add(index))
    }

    /// Construct a sequential iterator from the specified frame.
    ///
    /// # Safety
    ///
    /// The caller is responsible for ensuring that the lifetime of the produced
    /// iterator is valid.
    pub(crate) unsafe fn iter_from_frame<'a>(self, frame: usize) -> Iter<'a, T> {
        let data = slice::from_raw_parts(self.ptr.as_ptr() as *const _, self.len);
        let data = self
            .channels
            .checked_mul(frame)
            .and_then(|start| data.get(start..)?.get(..self.channels))
            .unwrap_or_default();
        Iter::new(data)
    }
}

/// The frame of a interleaved buffer.
pub struct InterleavedFrame<'a, T> {
    frame: usize,
    raw: RawInterleaved<T>,
    _marker: marker::PhantomData<&'a ()>,
}

impl<'a, T> InterleavedFrame<'a, T> {
    #[inline]
    pub(crate) fn new(frame: usize, raw: RawInterleaved<T>) -> Self {
        Self {
            frame,
            raw,
            _marker: marker::PhantomData,
        }
    }
}

impl<T> Frame for InterleavedFrame<'_, T>
where
    T: Copy,
{
    type Sample = T;

    type Frame<'this>
        = InterleavedFrame<'this, T>
    where
        Self: 'this;

    type Iter<'this>
        = Iter<'this, T>
    where
        Self: 'this;

    #[inline]
    fn as_frame(&self) -> Self::Frame<'_> {
        Self {
            frame: self.frame,
            raw: self.raw,
            _marker: marker::PhantomData,
        }
    }

    #[inline]
    fn len(&self) -> usize {
        self.raw.frames()
    }

    #[inline]
    fn get(&self, channel: usize) -> Option<Self::Sample> {
        // SAFETY: the constructor of this wrapper is unsafe and requires the
        // caller to guarantee its boundaries.
        unsafe { self.raw.get_sample(self.frame, channel) }
    }

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        // SAFETY: The construction of this buffer ensures that the iterator is
        // in line and the lifetime is bounded to the current object.
        unsafe { self.raw.iter_from_frame(self.frame) }
    }
}

/// An iterator over all frames.
pub struct InterleavedFramesIter<'a, T> {
    frame: usize,
    raw: RawInterleaved<T>,
    _marker: marker::PhantomData<&'a [T]>,
}

impl<'a, T> InterleavedFramesIter<'a, T> {
    #[inline]
    pub(crate) fn new(frame: usize, raw: RawInterleaved<T>) -> Self {
        Self {
            frame,
            raw,
            _marker: marker::PhantomData,
        }
    }
}

impl<'a, T> Iterator for InterleavedFramesIter<'a, T> {
    type Item = InterleavedFrame<'a, T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.frame >= self.raw.frames() {
            return None;
        }

        let frame = self.frame;
        self.frame = self.frame.checked_add(1)?;

        Some(InterleavedFrame {
            frame,
            raw: self.raw,
            _marker: marker::PhantomData,
        })
    }
}
