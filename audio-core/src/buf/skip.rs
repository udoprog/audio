use crate::buf::{Buf, ExactSizeBuf};
use crate::buf_mut::BufMut;
use crate::channel::Channel;
use crate::io::ReadBuf;

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
/// ```rust
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

    type Channel<'a>
    where
        Self::Sample: 'a,
    = B::Channel<'a>;

    fn frames_hint(&self) -> Option<usize> {
        let frames = self.buf.frames_hint()?;
        Some(frames.saturating_sub(self.n))
    }

    fn channels(&self) -> usize {
        self.buf.channels()
    }

    fn channel(&self, channel: usize) -> Self::Channel<'_> {
        self.buf.channel(channel).skip(self.n)
    }
}

impl<B> BufMut for Skip<B>
where
    B: BufMut,
{
    type ChannelMut<'a>
    where
        Self::Sample: 'a,
    = B::ChannelMut<'a>;

    fn channel_mut(&mut self, channel: usize) -> Self::ChannelMut<'_> {
        self.buf.channel_mut(channel).skip(self.n)
    }

    fn copy_channels(&mut self, from: usize, to: usize)
    where
        Self::Sample: Copy,
    {
        self.buf.copy_channels(from, to);
    }
}

/// [Skip] adjusts the implementation of [ExactSizeBuf].
///
/// ```rust
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
