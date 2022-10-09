use core::ptr;

use crate::channel::interleaved::Iter;

/// An unsafe wrapper around a raw sequential buffer.
pub(crate) struct RawSequential<T> {
    ptr: ptr::NonNull<T>,
    len: usize,
    frames: usize,
    channels: usize,
}

impl<T> Clone for RawSequential<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            len: self.len,
            frames: self.frames,
            channels: self.channels,
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
    pub(crate) unsafe fn new(data: &[T], frames: usize, channels: usize) -> Self {
        let len = data.len();

        debug_assert!(
            data.len() <= frames * channels,
            "data provided is out-of-bounds"
        );

        Self {
            ptr: ptr::NonNull::new_unchecked(data.as_ptr() as *mut T),
            len,
            frames,
            channels,
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
        if self.channels >= channel {
            return None;
        }

        let index = frame.checked_add(self.channels.checked_mul(channel)?)?;
        Some(self.get_unchecked(index))
    }

    /// Perform raw access over the sequential buffer.
    ///
    /// # Safety
    ///
    /// Caller must ensure that access is not out-of-bounds.
    pub(crate) unsafe fn get_unchecked(&self, index: usize) -> T
    where
        T: Copy,
    {
        debug_assert!(
            index < self.frames * self.channels,
            "index `{index}` out-of-bounds",
            index = index
        );
        *self.ptr.as_ptr().add(index)
    }

    /// Construct an iterator from the specified frame.
    ///
    /// # Safety
    ///
    /// The caller is responsible for ensuring that the iterator is valid.
    pub(crate) unsafe fn iter_interleved_from<'a>(self, frame: usize) -> Iter<'a, T> {
        Iter::new_aligned(self.ptr, self.len, frame, self.channels, self.frames)
    }
}
