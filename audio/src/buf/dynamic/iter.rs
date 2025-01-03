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
/// Created with [Dynamic::iter_channels][crate::buf::Dynamic::iter_channels].
pub struct IterChannels<'a, T> {
    iter: slice::Iter<'a, RawSlice<T>>,
    len: usize,
}

impl<'a, T> IterChannels<'a, T> {
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

impl<'a, T> Iterator for IterChannels<'a, T> {
    type Item = LinearChannel<'a, T>;

    forward!(as_linear_channel);
}

impl<T> DoubleEndedIterator for IterChannels<'_, T> {
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

impl<T> ExactSizeIterator for IterChannels<'_, T> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

/// A mutable iterator over the channels in the buffer.
///
/// Created with [Dynamic::iter_channels_mut][crate::buf::Dynamic::iter_channels_mut].
pub struct IterChannelsMut<'a, T> {
    iter: slice::IterMut<'a, RawSlice<T>>,
    len: usize,
}

impl<'a, T> IterChannelsMut<'a, T> {
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

impl<'a, T> Iterator for IterChannelsMut<'a, T> {
    type Item = LinearChannelMut<'a, T>;

    forward!(as_linear_channel_mut);
}

impl<T> DoubleEndedIterator for IterChannelsMut<'_, T> {
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

impl<T> ExactSizeIterator for IterChannelsMut<'_, T> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}
