use std::slice;

// Helper to forward slice-optimized iterator functions.
macro_rules! forward {
    () => {
        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            self.iter.next()
        }

        #[inline]
        fn nth(&mut self, n: usize) -> Option<Self::Item> {
            self.iter.nth(n)
        }

        #[inline]
        fn last(self) -> Option<Self::Item> {
            self.iter.last()
        }

        #[inline]
        fn find<P>(&mut self, predicate: P) -> Option<Self::Item>
        where
            Self: Sized,
            P: FnMut(&Self::Item) -> bool,
        {
            self.iter.find(predicate)
        }

        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>) {
            self.iter.size_hint()
        }

        #[inline]
        fn count(self) -> usize {
            self.iter.count()
        }

        #[inline]
        fn for_each<F>(self, f: F)
        where
            Self: Sized,
            F: FnMut(Self::Item),
        {
            self.iter.for_each(f);
        }

        #[inline]
        fn all<F>(&mut self, f: F) -> bool
        where
            Self: Sized,
            F: FnMut(Self::Item) -> bool,
        {
            self.iter.all(f)
        }

        #[inline]
        fn any<F>(&mut self, f: F) -> bool
        where
            Self: Sized,
            F: FnMut(Self::Item) -> bool,
        {
            self.iter.any(f)
        }

        #[inline]
        fn find_map<B, F>(&mut self, f: F) -> Option<B>
        where
            Self: Sized,
            F: FnMut(Self::Item) -> Option<B>,
        {
            self.iter.find_map(f)
        }

        #[inline]
        fn position<P>(&mut self, predicate: P) -> Option<usize>
        where
            Self: Sized,
            P: FnMut(Self::Item) -> bool,
        {
            self.iter.position(predicate)
        }

        #[inline]
        fn rposition<P>(&mut self, predicate: P) -> Option<usize>
        where
            P: FnMut(Self::Item) -> bool,
            Self: Sized,
        {
            self.iter.rposition(predicate)
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
    pub(super) fn new(data: &'a [T], frames: usize) -> Self {
        Self {
            iter: data.chunks_exact(frames),
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a [T];

    forward!();
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth_back(n)
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
    pub(super) fn new(data: &'a mut [T], frames: usize) -> Self {
        Self {
            iter: data.chunks_exact_mut(frames),
        }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut [T];

    forward!();
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth_back(n)
    }
}

impl<'a, T> ExactSizeIterator for IterMut<'a, T> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}
