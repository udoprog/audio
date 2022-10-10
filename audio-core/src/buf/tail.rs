use crate::{Buf, BufMut, Channel, ChannelMut, ExactSizeBuf, ReadBuf};

/// The tail of a buffer.
///
/// See [Buf::tail].
pub struct Tail<B> {
    buf: B,
    n: usize,
}

impl<B> Tail<B> {
    /// Construct a new buffer tail.
    #[inline]
    pub(crate) fn new(buf: B, n: usize) -> Self {
        Self { buf, n }
    }
}

impl<B> Buf for Tail<B>
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

    type Frame<'this> = B::Frame<'this>
    where
        Self: 'this;

    type IterFrames<'this> = B::IterFrames<'this>
    where
        Self: 'this;

    #[inline]
    fn frames_hint(&self) -> Option<usize> {
        let frames = self.buf.frames_hint()?;
        Some(usize::min(frames, self.n))
    }

    #[inline]
    fn channels(&self) -> usize {
        self.buf.channels()
    }

    #[inline]
    fn channel(&self, channel: usize) -> Option<Self::Channel<'_>> {
        Some(self.buf.channel(channel)?.tail(self.n))
    }

    #[inline]
    fn iter_channels(&self) -> Self::IterChannels<'_> {
        IterChannels {
            iter: self.buf.iter_channels(),
            n: self.n,
        }
    }

    #[inline]
    fn frame(&self, frame: usize) -> Option<Self::Frame<'_>> {
        self.buf.frame(frame)
    }

    #[inline]
    fn iter_frames(&self) -> Self::IterFrames<'_> {
        self.buf.iter_frames()
    }
}

impl<B> BufMut for Tail<B>
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
        Some(self.buf.channel_mut(channel)?.tail(self.n))
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
            n: self.n,
        }
    }
}

/// [Tail] adjusts the implementation of [ExactSizeBuf].
///
/// ```
/// use audio::{Buf, ExactSizeBuf};
///
/// let buf = audio::interleaved![[0; 4]; 2];
///
/// assert_eq!((&buf).tail(0).frames(), 0);
/// assert_eq!((&buf).tail(1).frames(), 1);
/// assert_eq!((&buf).tail(5).frames(), 4);
/// ```
impl<B> ExactSizeBuf for Tail<B>
where
    B: ExactSizeBuf,
{
    #[inline]
    fn frames(&self) -> usize {
        usize::min(self.buf.frames(), self.n)
    }
}

impl<B> ReadBuf for Tail<B>
where
    B: ReadBuf,
{
    #[inline]
    fn remaining(&self) -> usize {
        usize::min(self.buf.remaining(), self.n)
    }

    fn advance(&mut self, n: usize) {
        let n = self
            .buf
            .remaining()
            .saturating_sub(self.n)
            .saturating_add(n);

        self.buf.advance(n);
    }
}

iterators!(IterChannels, IterChannelsMut, n: usize => self.tail(n));
