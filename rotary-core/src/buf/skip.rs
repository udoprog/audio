use crate::buf::{Buf, Channels, ChannelsMut, ExactSizeBuf};
use crate::channel::{Channel, ChannelMut};
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
/// use rotary::{Buf, ExactSizeBuf};
///
/// let buf = rotary::interleaved![[0; 4]; 2];
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
    fn frames_hint(&self) -> Option<usize> {
        let frames = self.buf.frames_hint()?;
        Some(frames.saturating_sub(self.n))
    }

    fn channels(&self) -> usize {
        self.buf.channels()
    }
}

/// [Skip] adjusts the implementation of [ExactSizeBuf].
///
/// ```rust
/// use rotary::{Buf, ExactSizeBuf};
///
/// let buf = rotary::interleaved![[0; 4]; 2];
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

impl<B, T> Channels<T> for Skip<B>
where
    B: Channels<T>,
{
    fn channel(&self, channel: usize) -> Channel<'_, T> {
        self.buf.channel(channel).skip(self.n)
    }
}

impl<B, T> ChannelsMut<T> for Skip<B>
where
    B: ChannelsMut<T>,
{
    fn channel_mut(&mut self, channel: usize) -> ChannelMut<'_, T> {
        self.buf.channel_mut(channel).skip(self.n)
    }

    fn copy_channels(&mut self, from: usize, to: usize)
    where
        T: Copy,
    {
        self.buf.copy_channels(from, to);
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
