use std::iter;
use std::slice;

/// A channel slice iterator.
///
/// See [Channel::iter][crate::Channel::iter].
pub struct Iter<'a, T> {
    iter: iter::StepBy<slice::Iter<'a, T>>,
}

impl<'a, T> Iter<'a, T> {
    #[inline]
    pub(super) fn new(slice: &'a [T], step: usize) -> Self {
        Self {
            iter: slice.iter().step_by(step),
        }
    }
}

// Note: we include a bunch of forwarding implementations since they
impl<'a, T> Iterator for Iter<'a, T>
where
    T: Copy,
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().copied()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth(n).copied()
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        self.iter.last().copied()
    }

    #[inline]
    fn find<P>(&mut self, mut predicate: P) -> Option<Self::Item>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        self.iter.find(|item| predicate(*item)).copied()
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
        self.iter.for_each(move |item| f(*item));
    }

    #[inline]
    fn all<F>(&mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        self.iter.all(move |item| f(*item))
    }

    #[inline]
    fn any<F>(&mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        self.iter.any(move |item| f(*item))
    }

    #[inline]
    fn find_map<B, F>(&mut self, mut f: F) -> Option<B>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<B>,
    {
        self.iter.find_map(move |item| f(*item))
    }

    #[inline]
    fn position<P>(&mut self, mut predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        self.iter.position(move |item| predicate(*item))
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T>
where
    T: Copy,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().copied()
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth_back(n).copied()
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T>
where
    T: Copy,
{
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

/// A channel slice iterator.
///
/// See [ChannelMut::iter_mut][crate::ChannelMut::iter_mut].
pub struct IterMut<'a, T> {
    iter: iter::StepBy<slice::IterMut<'a, T>>,
}

impl<'a, T> IterMut<'a, T> {
    #[inline]
    pub(super) fn new(slice: &'a mut [T], step: usize) -> Self {
        Self {
            iter: slice.iter_mut().step_by(step),
        }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

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
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth_back(n)
    }
}

impl<'a, T> ExactSizeIterator for IterMut<'a, T> {
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}
