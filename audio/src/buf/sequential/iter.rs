use core::slice;

use crate::channel::{LinearChannel, LinearChannelMut};

// Helper to forward slice-optimized iterator functions.
macro_rules! forward {
    ($channel:ident) => {
        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            Some($channel::new(self.iter.next()?))
        }

        #[inline]
        fn nth(&mut self, n: usize) -> Option<Self::Item> {
            Some($channel::new(self.iter.nth(n)?))
        }

        #[inline]
        fn last(self) -> Option<Self::Item> {
            Some($channel::new(self.iter.last()?))
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
/// Created with [Sequential::iter_channels][super::Sequential::iter_channels].
pub struct IterChannels<'a, T> {
    iter: slice::ChunksExact<'a, T>,
}

impl<'a, T> IterChannels<'a, T> {
    #[inline]
    pub(crate) fn new(data: &'a [T], frames: usize) -> Self {
        Self {
            iter: data.chunks_exact(frames),
        }
    }
}

impl<'a, T> Iterator for IterChannels<'a, T> {
    type Item = LinearChannel<'a, T>;

    forward!(LinearChannel);
}

impl<T> DoubleEndedIterator for IterChannels<'_, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        Some(LinearChannel::new(self.iter.next_back()?))
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        Some(LinearChannel::new(self.iter.nth_back(n)?))
    }
}

impl<T> ExactSizeIterator for IterChannels<'_, T> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

/// A mutable iterator over the channels in the buffer.
///
/// Created with [Sequential::iter_channels_mut][super::Sequential::iter_channels_mut].
pub struct IterChannelsMut<'a, T> {
    iter: slice::ChunksExactMut<'a, T>,
}

impl<'a, T> IterChannelsMut<'a, T> {
    #[inline]
    pub(crate) fn new(data: &'a mut [T], frames: usize) -> Self {
        Self {
            iter: data.chunks_exact_mut(frames),
        }
    }
}

impl<'a, T> Iterator for IterChannelsMut<'a, T> {
    type Item = LinearChannelMut<'a, T>;

    forward!(LinearChannelMut);
}

impl<T> DoubleEndedIterator for IterChannelsMut<'_, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        Some(LinearChannelMut::new(self.iter.next_back()?))
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        Some(LinearChannelMut::new(self.iter.nth_back(n)?))
    }
}

impl<T> ExactSizeIterator for IterChannelsMut<'_, T> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}
