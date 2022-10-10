use crate::{Buf, BufMut, Channel, ChannelMut, ExactSizeBuf, ReadBuf};

/// A buffer that has been limited.
///
/// See [Buf::limit].
pub struct Limit<B> {
    buf: B,
    limit: usize,
}

impl<B> Limit<B> {
    /// Construct a new limited buffer.

    #[inline]
    pub(crate) fn new(buf: B, limit: usize) -> Self {
        Self { buf, limit }
    }
}

impl<B> Buf for Limit<B>
where
    B: Buf,
{
    type Sample = B::Sample;

    type Channel<'this> = B::Channel<'this>
    where
        Self: 'this;

    type IterChannels<'this> = IterChannels<B::IterChannels<'this>>
    where
        Self: 'this;

    #[inline]
    fn frames_hint(&self) -> Option<usize> {
        let frames = self.buf.frames_hint()?;
        Some(usize::min(frames, self.limit))
    }

    #[inline]
    fn channels(&self) -> usize {
        self.buf.channels()
    }

    #[inline]
    fn channel(&self, channel: usize) -> Option<Self::Channel<'_>> {
        Some(self.buf.channel(channel)?.limit(self.limit))
    }

    #[inline]
    fn iter_channels(&self) -> Self::IterChannels<'_> {
        IterChannels {
            iter: self.buf.iter_channels(),
            limit: self.limit,
        }
    }
}

impl<B> BufMut for Limit<B>
where
    B: BufMut,
{
    type ChannelMut<'this> = B::ChannelMut<'this>
    where
        Self: 'this;

    type IterChannelsMut<'this> = IterChannelsMut<B::IterChannelsMut<'this>>
    where
        Self: 'this;

    #[inline]
    fn channel_mut(&mut self, channel: usize) -> Option<Self::ChannelMut<'_>> {
        Some(self.buf.channel_mut(channel)?.limit(self.limit))
    }

    #[inline]
    fn copy_channel(&mut self, from: usize, to: usize)
    where
        Self::Sample: Copy,
    {
        self.buf.copy_channel(from, to);
    }

    #[inline]
    fn iter_channels_mut(&mut self) -> Self::IterChannelsMut<'_> {
        IterChannelsMut {
            iter: self.buf.iter_channels_mut(),
            limit: self.limit,
        }
    }
}

/// [Limit] adjusts the implementation of [ExactSizeBuf] to take the frame
/// limiting into account.
///
/// ```
/// use audio::{Buf, ExactSizeBuf};
///
/// let buf = audio::interleaved![[0; 4]; 2];
///
/// assert_eq!((&buf).limit(0).frames(), 0);
/// assert_eq!((&buf).limit(1).frames(), 1);
/// assert_eq!((&buf).limit(5).frames(), 4);
/// ```
impl<B> ExactSizeBuf for Limit<B>
where
    B: ExactSizeBuf,
{
    #[inline]
    fn frames(&self) -> usize {
        usize::min(self.buf.frames(), self.limit)
    }
}

impl<B> ReadBuf for Limit<B>
where
    B: ReadBuf,
{
    #[inline]
    fn remaining(&self) -> usize {
        usize::min(self.buf.remaining(), self.limit)
    }

    #[inline]
    fn advance(&mut self, n: usize) {
        self.buf.advance(usize::min(n, self.limit));
    }
}

iterators!(IterChannels, IterChannelsMut, limit: usize => self.limit(limit));
