//! Trait for dealing with abstract channel buffers.

use crate::Channel;

#[macro_use]
mod macros;

mod skip;
pub use self::skip::Skip;

mod limit;
pub use self::limit::Limit;

mod tail;
pub use self::tail::Tail;

/// The base trait available to all audio buffers.
///
/// This provides information which is available to all buffers, such as the
/// number of channels.
///
/// ```
/// let buf = audio::interleaved![[0; 4]; 2];
/// assert_eq!(buf.channels(), 2);
/// ```
///
/// It also carries a number of slicing combinators, wuch as [skip][Buf::skip]
/// and [limit][Buf::limit] which allows an audio buffer to be sliced as needed.
///
///
/// ```
/// use audio::{Buf, ExactSizeBuf};
///
/// let buf = audio::interleaved![[0; 4]; 2];
///
/// assert_eq!(buf.channels(), 2);
/// assert_eq!(buf.limit(2).frames(), 2);
/// ```
pub trait Buf {
    /// The type of a single sample.
    type Sample;

    /// The type of the channel container.
    type Channel<'this>: Channel<Sample = Self::Sample>
    where
        Self: 'this;

    /// An iterator over available channels.
    type Iter<'this>: Iterator<Item = Self::Channel<'this>>
    where
        Self: 'this;

    /// A typical number of frames for each channel in the buffer, if known.
    ///
    /// If you only want to support buffers which have exact sizes use
    /// [ExactSizeBuf][crate::ExactSizeBuf].
    ///
    /// This is only a best effort hint. We can't require any [Buf] to know the
    /// exact number of frames, because we want to be able to implement it for
    /// types which does not keep track of the exact number of frames it expects
    /// each channel to have such as `Vec<Vec<i16>>`.
    ///
    /// ```
    /// use audio::Buf;
    ///
    /// fn test(buf: impl Buf) {
    ///     assert_eq!(buf.channels(), 2);
    ///     assert_eq!(buf.frames_hint(), Some(4));
    /// }
    ///
    /// test(audio::wrap::dynamic(vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8]]));
    /// ```
    ///
    /// But it should be clear that such a buffer supports a variable number of
    /// frames in each channel.
    ///
    /// ```
    /// use audio::{Buf, Channel};
    ///
    /// fn test(buf: impl Buf<Sample = i16>) {
    ///     assert_eq!(buf.channels(), 2);
    ///     assert_eq!(buf.frames_hint(), Some(4));
    ///
    ///     assert_eq!(buf.get(0).map(|c| c.len()), Some(4));
    ///     assert_eq!(buf.get(1).map(|c| c.len()), Some(2));
    /// }
    ///
    /// test(audio::wrap::dynamic(vec![vec![1, 2, 3, 4], vec![5, 6]]));
    /// ```
    fn frames_hint(&self) -> Option<usize>;

    /// The number of channels in the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use audio::{Buf, Channel};
    ///
    /// fn test(buf: impl Buf<Sample = i16>) {
    ///     assert_eq!(buf.channels(), 2);
    ///
    ///     assert_eq! {
    ///         buf.get(0).unwrap().iter().collect::<Vec<_>>(),
    ///         &[1, 2, 3, 4],
    ///     }
    ///
    ///     assert_eq! {
    ///         buf.get(1).unwrap().iter().collect::<Vec<_>>(),
    ///         &[5, 6, 7, 8],
    ///     }
    /// }
    ///
    /// test(audio::interleaved![[1, 2, 3, 4], [5, 6, 7, 8]]);
    /// test(audio::wrap::interleaved(&[1, 5, 2, 6, 3, 7, 4, 8], 2));
    /// test(audio::wrap::dynamic(vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8]]));
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
    /// # Examples
    ///
    /// ```
    /// use audio::{Buf, Channel};
    ///
    /// fn test(buf: impl Buf<Sample = i16>) {
    ///     let chan = buf.get(1).unwrap();
    ///     chan.iter().eq([5, 6, 7, 8]);
    /// }
    ///
    /// test(audio::dynamic![[1, 2, 3, 4], [5, 6, 7, 8]]);
    /// test(audio::sequential![[1, 2, 3, 4], [5, 6, 7, 8]]);
    /// test(audio::interleaved![[1, 2, 3, 4], [5, 6, 7, 8]]);
    /// ```
    fn get(&self, channel: usize) -> Option<Self::Channel<'_>>;

    /// Construct an iterator over all the channels in the audio buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use audio::{Buf, Channel};
    ///
    /// fn test(buf: impl Buf<Sample = i16>) {
    ///     let chan = buf.iter().nth(1).unwrap();
    ///     chan.iter().eq([5, 6, 7, 8]);
    /// }
    ///
    /// test(audio::dynamic![[1, 2, 3, 4], [5, 6, 7, 8]]);
    /// test(audio::sequential![[1, 2, 3, 4], [5, 6, 7, 8]]);
    /// test(audio::interleaved![[1, 2, 3, 4], [5, 6, 7, 8]]);
    /// ```
    fn iter(&self) -> Self::Iter<'_>;

    /// Construct a new buffer where `n` frames are skipped.
    ///
    /// # Examples
    ///
    /// ```
    /// use audio::Buf;
    /// use audio::buf;
    ///
    /// let from = audio::interleaved![[0, 0, 1, 1], [0; 4]];
    /// let mut to = audio::buf::Interleaved::with_topology(2, 4);
    ///
    /// buf::copy(from.skip(2), &mut to);
    ///
    /// assert_eq!(to.as_slice(), &[1, 0, 1, 0, 0, 0, 0, 0]);
    /// ```
    ///
    /// With a mutable buffer.
    ///
    /// ```
    /// use audio::Buf;
    /// use audio::{buf, wrap};
    ///
    /// let from = wrap::interleaved(&[1, 1, 1, 1, 1, 1, 1, 1], 2);
    /// let mut to = audio::buf::Interleaved::with_topology(2, 4);
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
    /// ```
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
    ///
    /// The [tail][Buf::tail] of a buffer adjusts all functions associated with
    /// the [Buf]:
    ///
    /// ```
    /// use audio::{Buf, ExactSizeBuf};
    ///
    /// let buf = audio::interleaved![[1, 2, 3, 4]; 2];
    ///
    /// assert_eq!((&buf).tail(0).channels(), 2);
    /// assert_eq!((&buf).tail(0).frames_hint(), Some(0));
    ///
    /// assert_eq!((&buf).tail(1).channels(), 2);
    /// assert_eq!((&buf).tail(1).frames_hint(), Some(1));
    ///
    /// assert_eq!((&buf).tail(5).channels(), 2);
    /// assert_eq!((&buf).tail(5).frames_hint(), Some(4));
    ///
    /// for chan in buf.tail(2).iter() {
    ///     assert!(chan.iter().eq([3, 4]));
    /// }
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
    /// ```
    /// use audio::Buf;
    /// use audio::buf;
    ///
    /// let from = audio::interleaved![[1; 4]; 2];
    /// let mut to = audio::buf::Interleaved::with_topology(2, 4);
    ///
    /// buf::copy(from, (&mut to).limit(2));
    ///
    /// assert_eq!(to.as_slice(), &[1, 1, 1, 1, 0, 0, 0, 0]);
    /// ```
    ///
    /// The [limit][Buf::limit] of a buffer adjusts all functions associated
    /// with the [Buf]:
    ///
    /// ```
    /// use audio::Buf;
    ///
    /// let buf = audio::interleaved![[1, 2, 3, 4]; 2];
    ///
    /// assert_eq!((&buf).limit(0).channels(), 2);
    /// assert_eq!((&buf).limit(0).frames_hint(), Some(0));
    ///
    /// assert_eq!((&buf).limit(1).channels(), 2);
    /// assert_eq!((&buf).limit(1).frames_hint(), Some(1));
    ///
    /// assert_eq!((&buf).limit(5).channels(), 2);
    /// assert_eq!((&buf).limit(5).frames_hint(), Some(4));
    ///
    /// for chan in buf.limit(2).iter() {
    ///     assert!(chan.iter().eq([1, 2]));
    /// }
    /// ```
    fn limit(self, limit: usize) -> Limit<Self>
    where
        Self: Sized,
    {
        Limit::new(self, limit)
    }
}

impl<B> Buf for &B
where
    B: ?Sized + Buf,
{
    type Sample = B::Sample;

    type Channel<'a> = B::Channel<'a>
    where
        Self: 'a;

    type Iter<'a> = B::Iter<'a>
    where
        Self: 'a;

    #[inline]
    fn frames_hint(&self) -> Option<usize> {
        (**self).frames_hint()
    }

    #[inline]
    fn channels(&self) -> usize {
        (**self).channels()
    }

    #[inline]
    fn get(&self, channel: usize) -> Option<Self::Channel<'_>> {
        (**self).get(channel)
    }

    fn iter(&self) -> Self::Iter<'_> {
        (**self).iter()
    }
}

impl<B> Buf for &mut B
where
    B: ?Sized + Buf,
{
    type Sample = B::Sample;

    type Channel<'this> = B::Channel<'this>
    where
        Self: 'this;

    type Iter<'this> = B::Iter<'this>
    where
        Self: 'this;

    #[inline]
    fn frames_hint(&self) -> Option<usize> {
        (**self).frames_hint()
    }

    #[inline]
    fn channels(&self) -> usize {
        (**self).channels()
    }

    #[inline]
    fn get(&self, channel: usize) -> Option<Self::Channel<'_>> {
        (**self).get(channel)
    }

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        (**self).iter()
    }
}
