use crate::buf::dynamic::RawSlice;
use crate::channel::{LinearMut, LinearRef};
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
        fn find<P>(&mut self, mut predicate: P) -> Option<Self::Item>
        where
            Self: Sized,
            P: FnMut(&Self::Item) -> bool,
        {
            let len = self.len;
            let buf = self
                .iter
                .find(move |buf| predicate(&unsafe { buf.$as(len) }))?;
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

        #[inline]
        fn for_each<F>(self, mut f: F)
        where
            Self: Sized,
            F: FnMut(Self::Item),
        {
            let len = self.len;
            self.iter.for_each(move |buf| f(unsafe { buf.$as(len) }));
        }

        #[inline]
        fn all<F>(&mut self, mut f: F) -> bool
        where
            Self: Sized,
            F: FnMut(Self::Item) -> bool,
        {
            let len = self.len;
            self.iter.all(move |buf| f(unsafe { buf.$as(len) }))
        }

        #[inline]
        fn any<F>(&mut self, mut f: F) -> bool
        where
            Self: Sized,
            F: FnMut(Self::Item) -> bool,
        {
            let len = self.len;
            self.iter.any(move |buf| f(unsafe { buf.$as(len) }))
        }

        #[inline]
        fn find_map<B, F>(&mut self, mut f: F) -> Option<B>
        where
            Self: Sized,
            F: FnMut(Self::Item) -> Option<B>,
        {
            let len = self.len;
            self.iter.find_map(move |buf| f(unsafe { buf.$as(len) }))
        }

        #[inline]
        fn position<P>(&mut self, mut predicate: P) -> Option<usize>
        where
            Self: Sized,
            P: FnMut(Self::Item) -> bool,
        {
            let len = self.len;
            self.iter
                .position(move |buf| predicate(unsafe { buf.$as(len) }))
        }

        #[inline]
        fn rposition<P>(&mut self, mut predicate: P) -> Option<usize>
        where
            P: FnMut(Self::Item) -> bool,
            Self: Sized,
        {
            let len = self.len;
            self.iter
                .rposition(move |buf| predicate(unsafe { buf.$as(len) }))
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
    type Item = LinearRef<'a, T>;

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
    type Item = LinearMut<'a, T>;

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
