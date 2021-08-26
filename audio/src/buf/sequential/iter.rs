use crate::channel::{LinearMut, LinearRef};
use std::slice;

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
/// Created with [Sequential::iter][super::Sequential::iter].
pub struct Iter<'a, T> {
    iter: slice::ChunksExact<'a, T>,
}

impl<'a, T> Iter<'a, T> {
    #[inline]
    pub(crate) fn new(data: &'a [T], frames: usize) -> Self {
        Self {
            iter: data.chunks_exact(frames),
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = LinearRef<'a, T>;

    forward!(LinearRef);
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        Some(LinearRef::new(self.iter.next_back()?))
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        Some(LinearRef::new(self.iter.nth_back(n)?))
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

/// A mutable iterator over the channels in the buffer.
///
/// Created with [Sequential::iter_mut][super::Sequential::iter_mut].
pub struct IterMut<'a, T> {
    iter: slice::ChunksExactMut<'a, T>,
}

impl<'a, T> IterMut<'a, T> {
    #[inline]
    pub(crate) fn new(data: &'a mut [T], frames: usize) -> Self {
        Self {
            iter: data.chunks_exact_mut(frames),
        }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = LinearMut<'a, T>;

    forward!(LinearMut);
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        Some(LinearMut::new(self.iter.next_back()?))
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        Some(LinearMut::new(self.iter.nth_back(n)?))
    }
}

impl<'a, T> ExactSizeIterator for IterMut<'a, T> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}
