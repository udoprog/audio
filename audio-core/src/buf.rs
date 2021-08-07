//! Trait for dealing with abstract channel buffers.

use crate::channel::Channel;
use crate::linear_channel::LinearChannel;

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

/// The base trait available to all audio buffers.
///
/// This provides information which is available to all buffers, such as the
/// number of channels.
///
/// ```rust
/// use audio::Buf;
///
/// let buffer = audio::interleaved![[0; 4]; 2];
///
/// assert_eq!(buffer.channels(), 2);
/// ```
///
/// It also carries a number of slicing combinators, wuch as [skip][Buf::skip]
/// and [limit][Buf::limit] which allows an audio buffer to be sliced as needed.
///
///
/// ```rust
/// use audio::{Buf, ExactSizeBuf};
///
/// let buffer = audio::interleaved![[0; 4]; 2];
///
/// assert_eq!(buffer.channels(), 2);
/// assert_eq!(buffer.frames(), 4);
/// assert_eq!(buffer.limit(2).frames(), 2);
/// ```
pub trait Buf {
    /// The type of a single sample.
    type Sample;

    /// The type of the channel container.
    type Channel<'a>: Channel<Sample = Self::Sample>
    where
        Self::Sample: 'a;

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
    /// use audio::Buf;
    ///
    /// fn test(buf: impl Buf) {
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
    /// use audio::{Buf, Channel};
    ///
    /// fn test(buf: impl Buf<Sample = i16>) {
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
    /// use audio::{Buf, Channel};
    ///
    /// fn test(buf: impl Buf<Sample = i16>) {
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
    /// test(audio::interleaved![[1, 2, 3, 4], [5, 6, 7, 8]]);
    /// test(audio::wrap::interleaved(&[1, 5, 2, 6, 3, 7, 4, 8], 2));
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
    fn channel(&self, channel: usize) -> Self::Channel<'_>;

    /// Construct a new buffer where `n` frames are skipped.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::Buf;
    /// use audio::buf;
    ///
    /// let from = audio::interleaved![[0, 0, 1, 1], [0; 4]];
    /// let mut to = audio::Interleaved::with_topology(2, 4);
    ///
    /// buf::copy(from.skip(2), &mut to);
    ///
    /// assert_eq!(to.as_slice(), &[1, 0, 1, 0, 0, 0, 0, 0]);
    /// ```
    ///
    /// With a mutable buffer.
    ///
    /// ```rust
    /// use audio::Buf;
    /// use audio::{buf, wrap};
    ///
    /// let from = wrap::interleaved(&[1, 1, 1, 1, 1, 1, 1, 1], 2);
    /// let mut to = audio::Interleaved::with_topology(2, 4);
    ///
    /// buf::copy(from, (&mut to).skip(2));
    ///
    /// assert_eq!(to.as_slice(), &[0, 0, 0, 0, 1, 1, 1, 1])
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
    /// use audio::Buf;
    /// use audio::buf;
    ///
    /// let from = audio::interleaved![[1; 4]; 2];
    /// let mut to = audio::interleaved![[0; 4]; 2];
    ///
    /// buf::copy(from, (&mut to).tail(2));
    ///
    /// assert_eq!(to.as_slice(), &[0, 0, 0, 0, 1, 1, 1, 1]);
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
    /// use audio::Buf;
    /// use audio::buf;
    ///
    /// let from = audio::interleaved![[1; 4]; 2];
    /// let mut to = audio::Interleaved::with_topology(2, 4);
    ///
    /// buf::copy(from, (&mut to).limit(2));
    ///
    /// assert_eq!(to.as_slice(), &[1, 1, 1, 1, 0, 0, 0, 0]);
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
    /// use audio::Buf;
    /// use audio::buf;
    ///
    /// let from = audio::interleaved![[1; 4]; 2];
    /// let mut to = audio::interleaved![[0; 4]; 2];
    ///
    /// buf::copy(from, (&mut to).chunk(1, 2));
    ///
    /// assert_eq!(to.as_slice(), &[0, 0, 0, 0, 1, 1, 1, 1]);
    /// ```
    fn chunk(self, n: usize, len: usize) -> Chunk<Self>
    where
        Self: Sized,
    {
        Chunk::new(self, n, len)
    }
}

impl<B> Buf for &B
where
    B: ?Sized + Buf,
{
    type Sample = B::Sample;

    type Channel<'a>
    where
        Self::Sample: 'a,
    = B::Channel<'a>;

    #[inline]
    fn frames_hint(&self) -> Option<usize> {
        (**self).frames_hint()
    }

    #[inline]
    fn channels(&self) -> usize {
        (**self).channels()
    }

    #[inline]
    fn channel(&self, channel: usize) -> Self::Channel<'_> {
        (**self).channel(channel)
    }
}

impl<B> Buf for &mut B
where
    B: ?Sized + Buf,
{
    type Sample = B::Sample;

    type Channel<'a>
    where
        Self::Sample: 'a,
    = B::Channel<'a>;

    #[inline]
    fn frames_hint(&self) -> Option<usize> {
        (**self).frames_hint()
    }

    #[inline]
    fn channels(&self) -> usize {
        (**self).channels()
    }

    #[inline]
    fn channel(&self, channel: usize) -> Self::Channel<'_> {
        (**self).channel(channel)
    }
}

impl<T> Buf for Vec<Vec<T>> {
    type Sample = T;

    type Channel<'a>
    where
        Self::Sample: 'a,
    = LinearChannel<'a, T>;

    fn frames_hint(&self) -> Option<usize> {
        Some(self.get(0)?.len())
    }

    fn channels(&self) -> usize {
        self.len()
    }

    fn channel(&self, channel: usize) -> Self::Channel<'_> {
        LinearChannel::new(&self[channel])
    }
}

impl<T> Buf for [Vec<T>] {
    type Sample = T;

    type Channel<'a>
    where
        Self::Sample: 'a,
    = LinearChannel<'a, T>;

    fn frames_hint(&self) -> Option<usize> {
        Some(self.get(0)?.len())
    }

    fn channels(&self) -> usize {
        self.as_ref().len()
    }

    fn channel(&self, channel: usize) -> Self::Channel<'_> {
        LinearChannel::new(&self[channel])
    }
}
