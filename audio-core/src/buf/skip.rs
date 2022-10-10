use core::iter;

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
    #[inline]
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

    type Channel<'this> = B::Channel<'this>
    where
        Self: 'this;

    type IterChannels<'this> = IterChannels<B::IterChannels<'this>>
    where
        Self: 'this;

    type Frame<'this> = B::Frame<'this>
    where
        Self: 'this;

    type IterFrames<'this> = iter::Skip<B::IterFrames<'this>>
    where
        Self: 'this;

    #[inline]
    fn frames_hint(&self) -> Option<usize> {
        let frames = self.buf.frames_hint()?;
        Some(frames.saturating_sub(self.n))
    }

    #[inline]
    fn channels(&self) -> usize {
        self.buf.channels()
    }

    #[inline]
    fn channel(&self, channel: usize) -> Option<Self::Channel<'_>> {
        Some(self.buf.channel(channel)?.skip(self.n))
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
        self.buf.frame(frame.checked_add(self.n)?)
    }

    #[inline]
    fn iter_frames(&self) -> Self::IterFrames<'_> {
        self.buf.iter_frames().skip(self.n)
    }
}

impl<B> BufMut for Skip<B>
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
        Some(self.buf.channel_mut(channel)?.skip(self.n))
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
    #[inline]
    fn frames(&self) -> usize {
        self.buf.frames().saturating_sub(self.n)
    }
}

impl<B> ReadBuf for Skip<B>
where
    B: ReadBuf,
{
    #[inline]
    fn remaining(&self) -> usize {
        self.buf.remaining().saturating_sub(self.n)
    }

    #[inline]
    fn advance(&mut self, n: usize) {
        self.buf.advance(self.n.saturating_add(n));
    }
}

iterators!(IterChannels, IterChannelsMut, n: usize => self.skip(n));
