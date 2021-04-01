//! Trait for dealing with abstract channel buffers.

use crate::channel::{Channel, ChannelMut};

mod skip;
pub use self::skip::Skip;

mod limit;
pub use self::limit::Limit;

mod chunk;
pub use self::chunk::Chunk;

mod tail;
pub use self::tail::Tail;

mod exact_size_buf;
pub use self::exact_size_buf::ExactSizeBuf;

mod resizable_buf;
pub use self::resizable_buf::ResizableBuf;

mod interleaved_buf;
pub use self::interleaved_buf::InterleavedBuf;

mod as_interleaved;
pub use self::as_interleaved::AsInterleaved;

mod as_interleaved_mut;
pub use self::as_interleaved_mut::AsInterleavedMut;

/// A trait describing an immutable audio buffer.
pub trait Buf<T> {
    /// A typical number of frames for each channel in the buffer, if known.
    ///
    /// If you only want to support buffers which have exact sizes use
    /// [ExactSizeBuf].
    ///
    /// This is only a best effort hint. We can't require any [Buf] to know the
    /// exact number of frames, because we want to be able to implement it for
    /// types which does not keep track of the exact number of frames it expects
    /// each channel to have such as `Vec<Vec<i16>>`.
    ///
    /// ```rust
    /// use rotary::Buf;
    ///
    /// fn test(buf: impl Buf<i16>) {
    ///     assert_eq!(buf.channels(), 2);
    ///     assert_eq!(buf.frames_hint(), Some(4));
    /// }
    ///
    /// test(vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8]]);
    /// ```
    ///
    /// But it should be clear that such a buffer supports a variable number of
    /// frames in each channel.
    ///
    /// ```rust
    /// use rotary::Buf;
    ///
    /// fn test(buf: impl Buf<i16>) {
    ///     assert_eq!(buf.channels(), 2);
    ///     assert_eq!(buf.frames_hint(), Some(4));
    ///
    ///     assert_eq!(buf.channel(0).frames(), 4);
    ///     assert_eq!(buf.channel(1).frames(), 2);
    /// }
    ///
    /// test(vec![vec![1, 2, 3, 4], vec![5, 6]]);
    /// ```
    fn frames_hint(&self) -> Option<usize>;

    /// The number of channels in the buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::Buf;
    ///
    /// fn test(buf: impl Buf<i16>) {
    ///     assert_eq!(buf.channels(), 2);
    ///
    ///     assert_eq! {
    ///         buf.channel(0).iter().collect::<Vec<_>>(),
    ///         &[1, 2, 3, 4],
    ///     }
    ///
    ///     assert_eq! {
    ///         buf.channel(1).iter().collect::<Vec<_>>(),
    ///         &[5, 6, 7, 8],
    ///     }
    /// }
    ///
    /// test(rotary::interleaved![[1, 2, 3, 4], [5, 6, 7, 8]]);
    /// test(rotary::wrap::interleaved(&[1, 5, 2, 6, 3, 7, 4, 8], 2));
    /// test(vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8]]);
    /// ```
    fn channels(&self) -> usize;

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
    fn channel(&self, channel: usize) -> Channel<'_, T>;

    /// Construct a new buffer where `n` frames are skipped.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::{Buf as _, BufMut as _};
    ///
    /// let mut from = rotary::interleaved![[0.0f32; 4]; 2];
    /// *from.frame_mut(0, 2).unwrap() = 1.0;
    /// *from.frame_mut(0, 3).unwrap() = 1.0;
    ///
    /// let mut to = rotary::Interleaved::<f32>::with_topology(2, 4);
    ///
    /// to.channel_mut(0).copy_from((&from).skip(2).channel(0));
    /// assert_eq!(to.as_slice(), &[1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    /// ```
    ///
    /// Test with a mutable buffer.
    ///
    /// ```rust
    /// use rotary::{Buf as _, BufMut as _};
    ///
    /// let mut buffer = rotary::Interleaved::with_topology(2, 4);
    ///
    /// (&mut buffer).skip(2).channel_mut(0).copy_from_slice(&[1.0, 1.0]);
    ///
    /// assert_eq!(buffer.as_slice(), &[0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0])
    /// ```
    fn skip(self, n: usize) -> Skip<Self>
    where
        Self: Sized,
    {
        Skip::new(self, n)
    }

    /// Construct a new buffer where `n` frames are skipped.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::Buf as _;
    /// use rotary::buf;
    ///
    /// let from = rotary::interleaved![[1.0f32; 4]; 2];
    /// let mut to = rotary::interleaved![[0.0f32; 4]; 2];
    ///
    /// buf::copy(from, (&mut to).tail(2));
    ///
    /// assert_eq!(to.as_slice(), &[0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]);
    /// ```
    fn tail(self, n: usize) -> Tail<Self>
    where
        Self: Sized,
    {
        Tail::new(self, n)
    }

    /// Limit the channel buffer to `limit` number of frames.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::{Buf as _, BufMut as _};
    ///
    /// let from = rotary::interleaved![[1.0f32; 4]; 2];
    /// let mut to = rotary::Interleaved::<f32>::with_topology(2, 4);
    ///
    /// to.channel_mut(0).copy_from(from.limit(2).channel(0));
    /// assert_eq!(to.as_slice(), &[1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    /// ```
    fn limit(self, limit: usize) -> Limit<Self>
    where
        Self: Sized,
    {
        Limit::new(self, limit)
    }

    /// Construct a range of frames corresponds to the chunk with `len` and
    /// position `n`.
    ///
    /// Which is the range `n * len .. n * len + len`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::{Buf as _, BufMut as _};
    ///
    /// let from = rotary::interleaved![[1.0f32; 4]; 2];
    /// let mut to = rotary::interleaved![[0.0f32; 4]; 2];
    ///
    /// (&mut to).chunk(1, 2).channel_mut(0).copy_from(from.channel(0));
    /// assert_eq!(to.as_slice(), &[0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0]);
    /// ```
    fn chunk(self, n: usize, len: usize) -> Chunk<Self>
    where
        Self: Sized,
    {
        Chunk::new(self, n, len)
    }
}

/// A trait describing a mutable audio buffer.
pub trait BufMut<T>: Buf<T> {
    /// Return a mutable handler to the buffer associated with the channel.
    ///
    /// # Panics
    ///
    /// Panics if the specified channel is out of bound as reported by
    /// [Buf::channels].
    fn channel_mut(&mut self, channel: usize) -> ChannelMut<'_, T>;
}

impl<B, T> Buf<T> for &B
where
    B: Buf<T>,
{
    fn frames_hint(&self) -> Option<usize> {
        (**self).frames_hint()
    }

    fn channels(&self) -> usize {
        (**self).channels()
    }

    fn channel(&self, channel: usize) -> Channel<'_, T> {
        (**self).channel(channel)
    }
}

impl<B, T> Buf<T> for &mut B
where
    B: ?Sized + Buf<T>,
{
    fn frames_hint(&self) -> Option<usize> {
        (**self).frames_hint()
    }

    fn channels(&self) -> usize {
        (**self).channels()
    }

    fn channel(&self, channel: usize) -> Channel<'_, T> {
        (**self).channel(channel)
    }
}

impl<B, T> BufMut<T> for &mut B
where
    B: ?Sized + BufMut<T>,
{
    fn channel_mut(&mut self, channel: usize) -> ChannelMut<'_, T> {
        (**self).channel_mut(channel)
    }
}

impl<T> Buf<T> for Vec<Vec<T>> {
    fn frames_hint(&self) -> Option<usize> {
        Some(self.get(0)?.len())
    }

    fn channels(&self) -> usize {
        self.len()
    }

    fn channel(&self, channel: usize) -> Channel<'_, T> {
        Channel::linear(&self[channel])
    }
}

impl<T> BufMut<T> for Vec<Vec<T>> {
    fn channel_mut(&mut self, channel: usize) -> ChannelMut<'_, T> {
        ChannelMut::linear(&mut self[channel])
    }
}

impl<T> Buf<T> for [Vec<T>] {
    fn frames_hint(&self) -> Option<usize> {
        Some(self.get(0)?.len())
    }

    fn channels(&self) -> usize {
        self.as_ref().len()
    }

    fn channel(&self, channel: usize) -> Channel<'_, T> {
        Channel::linear(&self.as_ref()[channel])
    }
}
