use crate::{Buf, BufMut, Channel, ChannelMut, ExactSizeBuf, ReadBuf};

/// A buffer where a number of frames have been skipped over.
///
/// See [Buf::skip].
pub struct Skip<B> {
    buf: B,
    n: usize,
}

impl<B> Skip<B> {
    /// Construct a new buffer skip.
    pub(crate) fn new(buf: B, n: usize) -> Self {
        Self { buf, n }
    }
}

/// [Skip] adjusts the implementation of [Buf].
///
/// ```
/// use audio::{Buf, ExactSizeBuf};
///
/// let buf = audio::interleaved![[0; 4]; 2];
///
/// assert_eq!((&buf).skip(0).channels(), 2);
/// assert_eq!((&buf).skip(0).frames_hint(), Some(4));
///
/// assert_eq!((&buf).skip(1).channels(), 2);
/// assert_eq!((&buf).skip(1).frames_hint(), Some(3));
///
/// assert_eq!((&buf).skip(5).channels(), 2);
/// assert_eq!((&buf).skip(5).frames_hint(), Some(0));
/// ```
impl<B> Buf for Skip<B>
where
    B: Buf,
{
    type Sample = B::Sample;

    type Channel<'this>
        = B::Channel<'this>
    where
        Self: 'this;

    type IterChannels<'this>
        = IterChannels<B::IterChannels<'this>>
    where
        Self: 'this;

    fn frames_hint(&self) -> Option<usize> {
        let frames = self.buf.frames_hint()?;
        Some(frames.saturating_sub(self.n))
    }

    fn channels(&self) -> usize {
        self.buf.channels()
    }

    fn get_channel(&self, channel: usize) -> Option<Self::Channel<'_>> {
        Some(self.buf.get_channel(channel)?.skip(self.n))
    }

    fn iter_channels(&self) -> Self::IterChannels<'_> {
        IterChannels {
            iter: self.buf.iter_channels(),
            n: self.n,
        }
    }
}

impl<B> BufMut for Skip<B>
where
    B: BufMut,
{
    type ChannelMut<'a>
        = B::ChannelMut<'a>
    where
        Self: 'a;

    type IterChannelsMut<'a>
        = IterChannelsMut<B::IterChannelsMut<'a>>
    where
        Self: 'a;

    fn get_channel_mut(&mut self, channel: usize) -> Option<Self::ChannelMut<'_>> {
        Some(self.buf.get_channel_mut(channel)?.skip(self.n))
    }

    fn copy_channel(&mut self, from: usize, to: usize)
    where
        Self::Sample: Copy,
    {
        self.buf.copy_channel(from, to);
    }

    fn iter_channels_mut(&mut self) -> Self::IterChannelsMut<'_> {
        IterChannelsMut {
            iter: self.buf.iter_channels_mut(),
            n: self.n,
        }
    }
}

/// [Skip] adjusts the implementation of [ExactSizeBuf].
///
/// ```
/// use audio::{Buf, ExactSizeBuf};
///
/// let buf = audio::interleaved![[0; 4]; 2];
///
/// assert_eq!((&buf).skip(0).frames(), 4);
/// assert_eq!((&buf).skip(1).frames(), 3);
/// assert_eq!((&buf).skip(5).frames(), 0);
/// ```
impl<B> ExactSizeBuf for Skip<B>
where
    B: ExactSizeBuf,
{
    fn frames(&self) -> usize {
        self.buf.frames().saturating_sub(self.n)
    }
}

impl<B> ReadBuf for Skip<B>
where
    B: ReadBuf,
{
    fn remaining(&self) -> usize {
        self.buf.remaining().saturating_sub(self.n)
    }

    fn advance(&mut self, n: usize) {
        self.buf.advance(self.n.saturating_add(n));
    }
}

iterators!(n: usize => self.skip(n));
