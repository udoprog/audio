use crate::{Buf, ChannelMut};

/// A trait describing a mutable audio buffer.
pub trait BufMut: Buf {
    /// The type of the mutable channel container.
    type ChannelMut<'this>: ChannelMut<Sample = Self::Sample>
    where
        Self: 'this;

    /// A mutable iterator over available channels.
    type IterMut<'this>: Iterator<Item = Self::ChannelMut<'this>>
    where
        Self: 'this;

    /// Construct a mutable iterator over available channels.
    ///
    /// # Examples
    ///
    /// ```
    /// use audio::{BufMut, ChannelMut};
    ///
    /// fn test(mut buf: impl BufMut<Sample = i32>) {
    ///     for (n, mut chan) in buf.iter_mut().enumerate() {
    ///         for f in chan.iter_mut() {
    ///             *f += n as i32 + 1;
    ///         }
    ///     }
    /// }
    ///
    /// let mut buf = audio::dynamic![[0; 4]; 2];
    /// test(&mut buf);
    /// assert_eq!(
    ///     buf.iter().collect::<Vec<_>>(),
    ///     vec![[1, 1, 1, 1], [2, 2, 2, 2]],
    /// );
    ///
    /// let mut buf = audio::interleaved![[0; 4]; 2];
    /// test(&mut buf);
    /// assert_eq!(
    ///     buf.iter().collect::<Vec<_>>(),
    ///     vec![[1, 1, 1, 1], [2, 2, 2, 2]],
    /// );
    /// ```
    fn iter_mut(&mut self) -> Self::IterMut<'_>;

    /// Return a mutable handler to the buffer associated with the channel.
    ///
    /// # Examples
    ///
    /// ```
    /// use audio::{BufMut, ChannelMut};
    ///
    /// fn test(mut buf: impl BufMut<Sample = i32>) {
    ///     if let Some(mut chan) = buf.get_mut(1) {
    ///         for f in chan.iter_mut() {
    ///             *f += 1;
    ///         }
    ///     }
    /// }
    ///
    /// let mut buf = audio::dynamic![[0; 4]; 2];
    /// test(&mut buf);
    /// assert_eq!(
    ///     buf.iter().collect::<Vec<_>>(),
    ///     vec![[0, 0, 0, 0], [1, 1, 1, 1]],
    /// );
    /// ```
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
    /// ```
    /// use audio::{Buf, BufMut};
    ///
    /// let mut buf = audio::dynamic![[1, 2, 3, 4], [0, 0, 0, 0]];
    /// buf.copy_channel(0, 1);
    /// assert_eq!(buf.get(1), buf.get(0));
    /// ```
    fn copy_channel(&mut self, from: usize, to: usize)
    where
        Self::Sample: Copy;

    /// Fill the entire buffer with the specified value
    /// # Example
    ///
    /// ```
    /// use audio::BufMut;
    ///
    /// let mut buf = audio::sequential![[0; 2]; 2];
    /// buf.fill(1);
    /// assert_eq!(buf.as_slice(), &[1, 1, 1, 1]);
    /// ```
    fn fill(&mut self, value: Self::Sample)
    where
        Self::Sample: Copy,
    {
        for mut channel in self.iter_mut() {
            channel.fill(value);
        }
    }
}

impl<B> BufMut for &mut B
where
    B: ?Sized + BufMut,
{
    type ChannelMut<'this> = B::ChannelMut<'this>
    where
        Self: 'this;

    type IterMut<'this> = B::IterMut<'this>
    where
        Self: 'this;

    #[inline]
    fn get_mut(&mut self, channel: usize) -> Option<Self::ChannelMut<'_>> {
        (**self).get_mut(channel)
    }

    #[inline]
    fn copy_channel(&mut self, from: usize, to: usize)
    where
        Self::Sample: Copy,
    {
        (**self).copy_channel(from, to);
    }

    #[inline]
    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        (**self).iter_mut()
    }
}
