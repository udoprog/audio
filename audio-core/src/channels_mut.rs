use crate::channel_mut::ChannelMut;
use crate::channels::Channels;
use crate::linear_channel_mut::LinearChannelMut;

/// A trait describing a mutable audio buffer.
pub trait ChannelsMut: Channels {
    /// The type of the mutable channel container.
    type ChannelMut<'a>: ChannelMut<Sample = Self::Sample>
    where
        Self::Sample: 'a;

    /// Return a mutable handler to the buffer associated with the channel.
    ///
    /// # Panics
    ///
    /// Panics if the specified channel is out of bound as reported by
    /// [Buf::channels].
    fn channel_mut(&mut self, channel: usize) -> Self::ChannelMut<'_>;

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
    /// use audio::{Channels, ChannelsMut};
    ///
    /// let mut buffer: audio::Dynamic<i16> = audio::dynamic![[1, 2, 3, 4], [0, 0, 0, 0]];
    /// buffer.copy_channels(0, 1);
    ///
    /// assert_eq!(buffer.channel(1), buffer.channel(0));
    /// ```
    fn copy_channels(&mut self, from: usize, to: usize)
    where
        Self::Sample: Copy;
}

impl<B> ChannelsMut for &mut B
where
    B: ?Sized + ChannelsMut,
{
    type ChannelMut<'a>
    where
        Self::Sample: 'a,
    = B::ChannelMut<'a>;

    #[inline]
    fn channel_mut(&mut self, channel: usize) -> Self::ChannelMut<'_> {
        (**self).channel_mut(channel)
    }

    #[inline]
    fn copy_channels(&mut self, from: usize, to: usize)
    where
        Self::Sample: Copy,
    {
        (**self).copy_channels(from, to);
    }
}

impl<T> ChannelsMut for Vec<Vec<T>>
where
    T: Copy,
{
    type ChannelMut<'a>
    where
        Self::Sample: 'a,
    = LinearChannelMut<'a, T>;

    fn channel_mut(&mut self, channel: usize) -> Self::ChannelMut<'_> {
        LinearChannelMut::new(&mut self[channel])
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
}
