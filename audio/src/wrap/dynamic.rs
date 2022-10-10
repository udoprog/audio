use crate::channel::{LinearChannel, LinearChannelMut};
use audio_core::{Buf, BufMut, ResizableBuf, Sample};

/// A wrapper for an external dynamic audio buffer.
///
/// See [wrap::dynamic][super::dynamic()].
pub struct Dynamic<T> {
    value: T,
}

impl<T> Dynamic<T> {
    pub(crate) fn new(value: T) -> Self {
        Self { value }
    }

    /// Get a reference to the inner value.
    ///
    /// # Examples
    ///
    /// ```
    /// let buf = audio::wrap::dynamic(vec![vec![1, 2, 3, 4]]);
    /// assert_eq!(buf.as_ref(), &[vec![1, 2, 3, 4]]);
    /// ```
    #[inline]
    pub fn as_ref(&self) -> &T {
        &self.value
    }

    /// Get a mutable reference to the inner value.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut buf = audio::wrap::dynamic(vec![vec![1, 2, 3, 4]]);
    /// *buf.as_mut() = vec![vec![5, 6, 7, 8]];
    /// assert_eq!(buf.as_ref(), &[vec![5, 6, 7, 8]]);
    /// ```
    #[inline]
    pub fn as_mut(&mut self) -> &mut T {
        &mut self.value
    }

    /// Get the inner wrapper value.
    ///
    /// # Examples
    ///
    /// ```
    /// let buf = audio::wrap::dynamic(vec![vec![1, 2, 3, 4]]);
    /// assert_eq!(buf.into_inner(), vec![vec![1, 2, 3, 4]]);
    /// ```
    #[inline]
    pub fn into_inner(self) -> T {
        self.value
    }
}

macro_rules! impl_buf {
    ($ty:ty $(, $($extra:tt)*)?) => {
        impl<T $(, $($extra)*)*> Buf for Dynamic<$ty>
        where
            T: Copy,
        {
            type Sample = T;

            type Channel<'this> = LinearChannel<'this, Self::Sample>
            where
                Self: 'this;

            type Iter<'this> = Iter<'this, T>
            where
                Self: 'this;

            #[inline]
            fn frames_hint(&self) -> Option<usize> {
                Some(self.value.get(0)?.len())
            }

            #[inline]
            fn channels(&self) -> usize {
                self.value.len()
            }

            #[inline]
            fn get(&self, channel: usize) -> Option<Self::Channel<'_>> {
                Some(LinearChannel::new(self.value.get(channel)?))
            }

            #[inline]
            fn iter(&self) -> Self::Iter<'_> {
                Iter {
                    iter: self.value[..].iter(),
                }
            }
        }
    };
}

impl_buf!(Vec<Vec<T>>);
impl_buf!(&Vec<Vec<T>>);
impl_buf!(&mut Vec<Vec<T>>);
impl_buf!([Vec<T>; N], const N: usize);
impl_buf!(&[Vec<T>]);
impl_buf!(&[Vec<T>; N], const N: usize);
impl_buf!(&mut [Vec<T>]);
impl_buf!(&mut [Vec<T>; N], const N: usize);

macro_rules! impl_buf_mut {
    ($ty:ty $(, $($extra:tt)*)?) => {
        impl<T $(, $($extra)*)*> BufMut for Dynamic<$ty>
        where
            T: Copy,
        {
            type ChannelMut<'this> = LinearChannelMut<'this, T>
            where
                Self: 'this;

            type IterMut<'this> = IterMut<'this, T>
            where
                Self: 'this;

            #[inline]
            fn get_mut(&mut self, channel: usize) -> Option<Self::ChannelMut<'_>> {
                Some(LinearChannelMut::new(self.value.get_mut(channel)?.as_mut()))
            }

            fn copy_channel(&mut self, from: usize, to: usize) {
                assert! {
                    from < self.value.len(),
                    "copy from channel {} is out of bounds 0-{}",
                    from,
                    self.value.len()
                };
                assert! {
                    to < self.value.len(),
                    "copy to channel {} which is out of bounds 0-{}",
                    to,
                    self.value.len()
                };

                if from != to {
                    // Safety: We're making sure not to access any mutable buffers which are
                    // not initialized.
                    unsafe {
                        let ptr = self.value.as_mut_ptr();
                        let from = &*ptr.add(from);
                        let to = &mut *ptr.add(to);
                        let end = usize::min(from.len(), to.len());
                        to[..end].copy_from_slice(&from[..end]);
                    }
                }
            }

            #[inline]
            fn iter_mut(&mut self) -> Self::IterMut<'_> {
                IterMut {
                    iter: self.value[..].iter_mut(),
                }
            }
        }
    };
}

impl_buf_mut!(Vec<Vec<T>>);
impl_buf_mut!([Vec<T>; N], const N: usize);
impl_buf_mut!(&mut Vec<Vec<T>>);
impl_buf_mut!(&mut [Vec<T>]);
impl_buf_mut!(&mut [Vec<T>; N], const N: usize);

impl<T> ResizableBuf for Dynamic<Vec<Vec<T>>>
where
    T: Sample,
{
    #[inline]
    fn try_reserve(&mut self, _capacity: usize) -> bool {
        false
    }

    #[inline]
    fn resize(&mut self, frames: usize) {
        for buf in self.value.iter_mut() {
            buf.resize(frames, T::ZERO);
        }
    }

    fn resize_topology(&mut self, channels: usize, frames: usize) {
        for buf in self.value.iter_mut() {
            buf.resize(frames, T::ZERO);
        }

        for _ in self.value.len()..channels {
            self.value.push(vec![T::ZERO; frames]);
        }
    }
}

/// An iterator over a linear channel slice buffer.
pub struct Iter<'a, T> {
    iter: std::slice::Iter<'a, Vec<T>>,
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: Copy,
{
    type Item = LinearChannel<'a, T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        Some(LinearChannel::new(self.iter.next()?))
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        Some(LinearChannel::new(self.iter.nth(n)?))
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T>
where
    T: Copy,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        Some(LinearChannel::new(self.iter.next_back()?))
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        Some(LinearChannel::new(self.iter.nth_back(n)?))
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

/// A mutable iterator over a linear channel slice buffer.
pub struct IterMut<'a, T> {
    iter: std::slice::IterMut<'a, Vec<T>>,
}

impl<'a, T> Iterator for IterMut<'a, T>
where
    T: Copy,
{
    type Item = LinearChannelMut<'a, T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        Some(LinearChannelMut::new(self.iter.next()?))
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        Some(LinearChannelMut::new(self.iter.nth(n)?))
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T>
where
    T: Copy,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        Some(LinearChannelMut::new(self.iter.next_back()?))
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        Some(LinearChannelMut::new(self.iter.nth_back(n)?))
    }
}

impl<'a, T> ExactSizeIterator for IterMut<'a, T>
where
    T: Copy,
{
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}
