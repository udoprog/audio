use crate::buf::{Buf, Channels, ChannelsMut, ExactSizeBuf};
use crate::channel::{Channel, ChannelMut};
use crate::io::ReadBuf;

/// The tail of a buffer.
///
/// See [Buf::tail].
pub struct Tail<B> {
    buf: B,
    n: usize,
}

impl<B> Tail<B> {
    /// Construct a new buffer tail.
    pub(crate) fn new(buf: B, n: usize) -> Self {
        Self { buf, n }
    }
}

/// [Tail] adjusts the implementation of [Buf].
///
/// ```rust
/// use audio::{Buf, ExactSizeBuf};
///
/// let buf = audio::interleaved![[0; 4]; 2];
///
/// assert_eq!((&buf).tail(0).channels(), 2);
/// assert_eq!((&buf).tail(0).frames_hint(), Some(0));
///
/// assert_eq!((&buf).tail(1).channels(), 2);
/// assert_eq!((&buf).tail(1).frames_hint(), Some(1));
///
/// assert_eq!((&buf).tail(5).channels(), 2);
/// assert_eq!((&buf).tail(5).frames_hint(), Some(4));
/// ```
impl<B> Buf for Tail<B>
where
    B: Buf,
{
    fn frames_hint(&self) -> Option<usize> {
        let frames = self.buf.frames_hint()?;
        Some(usize::min(frames, self.n))
    }

    fn channels(&self) -> usize {
        self.buf.channels()
    }
}

/// [Tail] adjusts the implementation of [ExactSizeBuf].
///
/// ```rust
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
    fn frames(&self) -> usize {
        usize::min(self.buf.frames(), self.n)
    }
}

impl<B, T> Channels<T> for Tail<B>
where
    B: Channels<T>,
{
    fn channel(&self, channel: usize) -> Channel<'_, T> {
        self.buf.channel(channel).tail(self.n)
    }
}

impl<B, T> ChannelsMut<T> for Tail<B>
where
    B: ChannelsMut<T>,
{
    fn channel_mut(&mut self, channel: usize) -> ChannelMut<'_, T> {
        self.buf.channel_mut(channel).tail(self.n)
    }

    fn copy_channels(&mut self, from: usize, to: usize)
    where
        T: Copy,
    {
        self.buf.copy_channels(from, to);
    }
}

impl<B> ReadBuf for Tail<B>
where
    B: ReadBuf,
{
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
