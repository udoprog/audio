use crate::buf::{Buf, Channels, ChannelsMut, ExactSizeBuf};
use crate::channel::{Channel, ChannelMut};
use crate::io::ReadBuf;

/// A buffer that has been limited.
///
/// See [Buf::limit].
pub struct Limit<B> {
    buf: B,
    limit: usize,
}

impl<B> Limit<B> {
    /// Construct a new limited buffer.
    pub(crate) fn new(buf: B, limit: usize) -> Self {
        Self { buf, limit }
    }
}

/// [Limit] adjusts various implementations to report sensible values, such
/// as [Buf].
///
/// ```rust
/// use audio::Buf;
///
/// let buf = audio::interleaved![[0; 4]; 2];
///
/// assert_eq!((&buf).limit(0).channels(), 2);
/// assert_eq!((&buf).limit(0).frames_hint(), Some(0));
///
/// assert_eq!((&buf).limit(1).channels(), 2);
/// assert_eq!((&buf).limit(1).frames_hint(), Some(1));
///
/// assert_eq!((&buf).limit(5).channels(), 2);
/// assert_eq!((&buf).limit(5).frames_hint(), Some(4));
/// ```
impl<B> Buf for Limit<B>
where
    B: Buf,
{
    fn frames_hint(&self) -> Option<usize> {
        let frames = self.buf.frames_hint()?;
        Some(usize::min(frames, self.limit))
    }

    fn channels(&self) -> usize {
        self.buf.channels()
    }
}

/// [Limit] adjusts the implementation of [ExactSizeBuf] to take the frame
/// limiting into account.
///
/// ```rust
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
    fn frames(&self) -> usize {
        usize::min(self.buf.frames(), self.limit)
    }
}

impl<B, T> Channels<T> for Limit<B>
where
    B: Channels<T>,
{
    fn channel(&self, channel: usize) -> Channel<'_, T> {
        self.buf.channel(channel).limit(self.limit)
    }
}

impl<B, T> ChannelsMut<T> for Limit<B>
where
    B: ChannelsMut<T>,
{
    fn channel_mut(&mut self, channel: usize) -> ChannelMut<'_, T> {
        self.buf.channel_mut(channel).limit(self.limit)
    }

    fn copy_channels(&mut self, from: usize, to: usize)
    where
        T: Copy,
    {
        self.buf.copy_channels(from, to);
    }
}

impl<B> ReadBuf for Limit<B>
where
    B: ReadBuf,
{
    fn remaining(&self) -> usize {
        usize::min(self.buf.remaining(), self.limit)
    }

    fn advance(&mut self, n: usize) {
        self.buf.advance(usize::min(n, self.limit));
    }
}
