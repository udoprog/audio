use crate::{Buf, ChannelMut};

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
    /// buf.copy_channels(0, 1);
    /// assert_eq!(buf.get(1), buf.get(0));
    /// ```
    fn copy_channels(&mut self, from: usize, to: usize)
    where
        Self::Sample: Copy;
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
