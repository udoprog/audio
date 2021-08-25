use crate::{Buf, ChannelMut, LinearMut};

/// A trait describing a mutable audio buffer.
pub trait BufMut: Buf {
    /// The type of the mutable channel container.
    type ChannelMut<'a>: ChannelMut<Sample = Self::Sample>
    where
        Self::Sample: 'a;

    /// A mutable iterator over available channels.
    type IterMut<'a>: Iterator<Item = Self::ChannelMut<'a>>
    where
        Self::Sample: 'a;

    /// Return a mutable handler to the buffer associated with the channel.
    fn get_mut(&mut self, channel: usize) -> Option<Self::ChannelMut<'_>>;

    /// Copy one channel into another.
    ///
    /// If the channels have different sizes, the minimul difference between
    /// them will be copied.
    ///
    /// # Panics
    ///
    /// Panics if one of the channels being tried to copy from or to is out of
    /// bounds as reported by [Buf::channels].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::{Buf, BufMut};
    ///
    /// let mut buffer: audio::buf::Dynamic<i16> = audio::dynamic![[1, 2, 3, 4], [0, 0, 0, 0]];
    /// buffer.copy_channels(0, 1);
    ///
    /// assert_eq!(buffer.get(1), buffer.get(0));
    /// ```
    fn copy_channels(&mut self, from: usize, to: usize)
    where
        Self::Sample: Copy;

    /// Construct a mutable iterator over available channels.
    fn iter_mut(&mut self) -> Self::IterMut<'_>;
}

impl<B> BufMut for &mut B
where
    B: ?Sized + BufMut,
{
    type ChannelMut<'a>
    where
        Self::Sample: 'a,
    = B::ChannelMut<'a>;

    type IterMut<'a>
    where
        Self::Sample: 'a,
    = B::IterMut<'a>;

    #[inline]
    fn get_mut(&mut self, channel: usize) -> Option<Self::ChannelMut<'_>> {
        (**self).get_mut(channel)
    }

    #[inline]
    fn copy_channels(&mut self, from: usize, to: usize)
    where
        Self::Sample: Copy,
    {
        (**self).copy_channels(from, to);
    }

    #[inline]
    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        (**self).iter_mut()
    }
}

impl<T> BufMut for Vec<Vec<T>>
where
    T: Copy,
{
    type ChannelMut<'a>
    where
        Self::Sample: 'a,
    = LinearMut<'a, T>;

    type IterMut<'a>
    where
        Self::Sample: 'a,
    = VecIterMut<'a, T>;

    fn get_mut(&mut self, channel: usize) -> Option<Self::ChannelMut<'_>> {
        Some(LinearMut::new((**self).get_mut(channel)?.as_mut()))
    }

    fn copy_channels(&mut self, from: usize, to: usize) {
        assert! {
            from < self.len(),
            "copy from channel {} is out of bounds 0-{}",
            from,
            self.len()
        };
        assert! {
            to < self.len(),
            "copy to channel {} which is out of bounds 0-{}",
            to,
            self.len()
        };

        if from != to {
            // Safety: We're making sure not to access any mutable buffers which are
            // not initialized.
            unsafe {
                let ptr = self.as_mut_ptr();
                let from = &*ptr.add(from);
                let to = &mut *ptr.add(to);
                let end = usize::min(from.len(), to.len());
                to[..end].copy_from_slice(&from[..end]);
            }
        }
    }

    #[inline]
    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        VecIterMut {
            iter: (**self).iter_mut(),
        }
    }
}

/// A mutable iterator over a linear channel slice buffer.
pub struct VecIterMut<'a, T> {
    iter: std::slice::IterMut<'a, Vec<T>>,
}

impl<'a, T> Iterator for VecIterMut<'a, T>
where
    T: Copy,
{
    type Item = LinearMut<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(LinearMut::new(self.iter.next()?))
    }
}
