use crate::buf::Buf;
use crate::channel::Channel;
use crate::linear_channel::LinearChannel;

/// A trait describing something that has channels.
pub trait Channels: Buf {
    /// The type of a single sample.
    type Sample;

    /// The type of the channel container.
    type Channel<'a>: Channel<Sample = Self::Sample>
    where
        Self::Sample: 'a;

    /// Return a handler to the buffer associated with the channel.
    ///
    /// Note that we don't access the buffer for the underlying channel directly
    /// as a linear buffer like `&[T]`, because the underlying representation
    /// might be different.
    ///
    /// We must instead make use of the various utility functions found on
    /// [Channel] to copy data out of the channel.
    ///
    /// # Panics
    ///
    /// Panics if the specified channel is out of bound as reported by
    /// [Buf::channels].
    fn channel(&self, channel: usize) -> Self::Channel<'_>;
}

impl<B> Channels for &B
where
    B: ?Sized + Channels,
{
    type Sample = B::Sample;

    type Channel<'a>
    where
        Self::Sample: 'a,
    = B::Channel<'a>;

    #[inline]
    fn channel(&self, channel: usize) -> Self::Channel<'_> {
        (**self).channel(channel)
    }
}

impl<B> Channels for &mut B
where
    B: ?Sized + Channels,
{
    type Sample = B::Sample;

    type Channel<'a>
    where
        Self::Sample: 'a,
    = B::Channel<'a>;

    #[inline]
    fn channel(&self, channel: usize) -> Self::Channel<'_> {
        (**self).channel(channel)
    }
}

impl<T> Channels for Vec<Vec<T>>
where
    T: Copy,
{
    type Sample = T;

    type Channel<'a>
    where
        Self::Sample: 'a,
    = LinearChannel<'a, T>;

    fn channel(&self, channel: usize) -> Self::Channel<'_> {
        LinearChannel::new(&self[channel])
    }
}

impl<T> Channels for [Vec<T>] {
    type Sample = T;

    type Channel<'a>
    where
        Self::Sample: 'a,
    = LinearChannel<'a, T>;

    fn channel(&self, channel: usize) -> Self::Channel<'_> {
        LinearChannel::new(&self[channel])
    }
}
