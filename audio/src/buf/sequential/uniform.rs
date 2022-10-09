use core::marker;

use crate::buf::sequential::RawSequential;
use crate::channel::interleaved::Iter;
use audio_core::Frame;

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
        unsafe { self.raw.iter_interleved_from(self.frame) }
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
