use crate::buf::dynamic::RawSlice;
use crate::channel::{LinearChannel, LinearChannelMut};
use std::slice;

// Helper to forward slice-optimized iterator functions.
macro_rules! forward {
    ($as:ident) => {
        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            let buf = self.iter.next()?;
            Some(unsafe { buf.$as(self.len) })
        }

        #[inline]
        fn nth(&mut self, n: usize) -> Option<Self::Item> {
            let buf = self.iter.nth(n)?;
            Some(unsafe { buf.$as(self.len) })
        }

        #[inline]
        fn last(self) -> Option<Self::Item> {
            let buf = self.iter.last()?;
            Some(unsafe { buf.$as(self.len) })
        }

        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>) {
            self.iter.size_hint()
        }

        #[inline]
        fn count(self) -> usize {
            self.iter.count()
        }
    };
}

/// An iterator over the channels in the buffer.
///
/// Created with [Dynamic::iter][crate::buf::Dynamic::iter].
pub struct Iter<'a, T> {
    iter: slice::Iter<'a, RawSlice<T>>,
    len: usize,
}

impl<'a, T> Iter<'a, T> {
    /// Construct a new iterator.
    ///
    /// # Safety
    ///
    /// The provided `len` must match the lengths of all provided slices.
    #[inline]
    pub(super) unsafe fn new(data: &'a [RawSlice<T>], len: usize) -> Self {
        Self {
            iter: data.iter(),
            len,
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = LinearChannel<'a, T>;

    forward!(as_linear_channel);
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let buf = self.iter.next_back()?;
        Some(unsafe { buf.as_linear_channel(self.len) })
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let buf = self.iter.nth_back(n)?;
        Some(unsafe { buf.as_linear_channel(self.len) })
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

/// A mutable iterator over the channels in the buffer.
///
/// Created with [Dynamic::iter_mut][crate::buf::Dynamic::iter_mut].
pub struct IterMut<'a, T> {
    iter: slice::IterMut<'a, RawSlice<T>>,
    len: usize,
}

impl<'a, T> IterMut<'a, T> {
    /// Construct a new iterator.
    ///
    /// # Safety
    ///
    /// The provided `len` must match the lengths of all provided slices.
    #[inline]
    pub(super) unsafe fn new(data: &'a mut [RawSlice<T>], len: usize) -> Self {
        Self {
            iter: data.iter_mut(),
            len,
        }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = LinearChannelMut<'a, T>;

    forward!(as_linear_channel_mut);
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let buf = self.iter.next_back()?;
        Some(unsafe { buf.as_linear_channel_mut(self.len) })
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let buf = self.iter.nth_back(n)?;
        Some(unsafe { buf.as_linear_channel_mut(self.len) })
    }
}

impl<'a, T> ExactSizeIterator for IterMut<'a, T> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}
