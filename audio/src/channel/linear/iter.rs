use core::slice;

/// An iterator over the frames in a linear channel.
///
/// Created with [LinearChannel::iter][super::LinearChannel::iter].
pub struct Iter<'a, T> {
    iter: slice::Iter<'a, T>,
}

impl<'a, T> Iter<'a, T> {
    #[inline]
    pub(crate) fn new(data: &'a [T]) -> Self {
        Self { iter: data.iter() }
    }

    /// Views the underlying data as a subslice of the original data.
    #[inline]
    pub fn as_slice(&self) -> &'a [T] {
        self.iter.as_slice()
    }
}

impl<T> Iterator for Iter<'_, T>
where
    T: Copy,
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        Some(*self.iter.next()?)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        Some(*self.iter.nth(n)?)
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        Some(*self.iter.last()?)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.iter.count()
    }
}

impl<T> DoubleEndedIterator for Iter<'_, T>
where
    T: Copy,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        Some(*self.iter.next_back()?)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        Some(*self.iter.nth_back(n)?)
    }
}

impl<T> ExactSizeIterator for Iter<'_, T>
where
    T: Copy,
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

/// A mutable iterator over the frames in a linear channel.
///
/// Created with [LinearChannelMut::iter_mut][super::LinearChannelMut::iter_mut].
pub struct IterMut<'a, T> {
    iter: slice::IterMut<'a, T>,
}

impl<'a, T> IterMut<'a, T> {
    #[inline]
    pub(crate) fn new(data: &'a mut [T]) -> Self {
        Self {
            iter: data.iter_mut(),
        }
    }

    /// Views the underlying data as a subslice of the original data.
    ///
    /// To avoid creating `&mut` references that alias, this is forced to
    /// consume the iterator.
    pub fn into_slice(self) -> &'a [T] {
        self.iter.into_slice()
    }

    /// Views the underlying data as a subslice of the original data.
    pub fn as_slice(&self) -> &[T] {
        self.iter.as_slice()
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

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
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.iter.count()
    }
}

impl<T> DoubleEndedIterator for IterMut<'_, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth_back(n)
    }
}

impl<T> ExactSizeIterator for IterMut<'_, T> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}
