use core::marker;
use core::ptr;

use audio_core::Frame;

use crate::channel::interleaved::Iter;

/// An unsafe wrapper around a raw sequential buffer.
pub(crate) struct RawSequential<T> {
    ptr: ptr::NonNull<T>,
    len: usize,
    channels: usize,
    frames: usize,
}

impl<T> Clone for RawSequential<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            len: self.len,
            channels: self.channels,
            frames: self.frames,
        }
    }
}

impl<T> Copy for RawSequential<T> {}

impl<T> RawSequential<T> {
    /// Construct a new raw sequential buffer.
    ///
    /// # Safety
    ///
    /// Caller must ensure that the provided buffer is within bounds of the given specification.
    pub(crate) unsafe fn new(data: &[T], channels: usize, frames: usize) -> Self {
        let len = data.len();

        debug_assert!(
            len % channels == 0,
            "data provided is not aligned with the number of channels; len={}, channels={}",
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
        let index = frame.checked_add(self.frames.checked_mul(channel)?)?;

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

    /// Construct an interleaved iterator from the specified frame.
    ///
    /// # Safety
    ///
    /// The caller is responsible for ensuring that the lifetime of the produced
    /// iterator is valid and that the specified frame is in bounds.
    pub(crate) unsafe fn iter_from_frame<'a>(self, frame: usize) -> Iter<'a, T> {
        debug_assert!(
            frame < self.frames,
            "frame out of bounds; frame={}, frames={}",
            frame,
            self.frames
        );
        Iter::new_aligned(self.ptr, self.len, frame, self.channels, self.frames)
    }
}

/// The frame of a sequential buffer.
pub struct SequentialFrame<'a, T> {
    frame: usize,
    raw: RawSequential<T>,
    _marker: marker::PhantomData<&'a ()>,
}

impl<'a, T> SequentialFrame<'a, T> {
    #[inline]
    pub(crate) fn new(frame: usize, raw: RawSequential<T>) -> Self {
        Self {
            frame,
            raw,
            _marker: marker::PhantomData,
        }
    }
}

impl<'a, T> Frame for SequentialFrame<'a, T>
where
    T: Copy,
{
    type Sample = T;

    type Frame<'this> = SequentialFrame<'this, T>
    where
        Self: 'this;

    type Iter<'this> = Iter<'this, T>
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
    fn get(&self, n: usize) -> Option<Self::Sample> {
        // SAFETY: the constructor of this wrapper is unsafe and requires the
        // caller to guarantee its boundaries.
        unsafe { self.raw.get_sample(self.frame, n) }
    }

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        // SAFETY: The construction of this buffer ensures that the iterator is
        // in line and the lifetime is bounded to the current object.
        unsafe { self.raw.iter_from_frame(self.frame) }
    }
}

/// An iterator over all frames.
pub struct SequentialFramesIter<'a, T> {
    frame: usize,
    raw: RawSequential<T>,
    _marker: marker::PhantomData<&'a [T]>,
}

impl<'a, T> SequentialFramesIter<'a, T> {
    #[inline]
    pub(crate) fn new(frame: usize, raw: RawSequential<T>) -> Self {
        Self {
            frame,
            raw,
            _marker: marker::PhantomData,
        }
    }
}

impl<'a, T> Iterator for SequentialFramesIter<'a, T> {
    type Item = SequentialFrame<'a, T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.frame >= self.raw.frames() {
            return None;
        }

        let frame = self.frame;
        self.frame = self.frame.checked_add(1)?;

        Some(SequentialFrame {
            frame,
            raw: self.raw,
            _marker: marker::PhantomData,
        })
    }
}
