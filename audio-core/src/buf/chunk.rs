use crate::{Buf, BufMut, Channel, ExactSizeBuf};

/// A chunk of another buffer.
///
/// See [Buf::chunk].
pub struct Chunk<B> {
    buf: B,
    n: usize,
    len: usize,
}

impl<B> Chunk<B> {
    /// Construct a new limited buffer.
    pub(crate) fn new(buf: B, n: usize, len: usize) -> Self {
        Self { buf, n, len }
    }
}

/// ```rust
/// use audio::Buf;
///
/// let buf = audio::interleaved![[0; 4]; 2];
///
/// assert_eq!((&buf).chunk(0, 3).channels(), 2);
/// assert_eq!((&buf).chunk(0, 3).frames_hint(), Some(3));
///
/// assert_eq!((&buf).chunk(1, 3).channels(), 2);
/// assert_eq!((&buf).chunk(1, 3).frames_hint(), Some(1));
///
/// assert_eq!((&buf).chunk(2, 3).channels(), 2);
/// assert_eq!((&buf).chunk(2, 3).frames_hint(), Some(0));
/// ```
impl<B> Buf for Chunk<B>
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
        let len = frames.saturating_sub(self.n.saturating_mul(self.len));
        Some(usize::min(len, self.len))
    }

    fn channels(&self) -> usize {
        self.buf.channels()
    }

    fn channel(&self, channel: usize) -> Self::Channel<'_> {
        self.buf.channel(channel).chunk(self.n, self.len)
    }
}

impl<B> BufMut for Chunk<B>
where
    B: BufMut,
{
    type ChannelMut<'a>
    where
        Self::Sample: 'a,
    = B::ChannelMut<'a>;

    fn channel_mut(&mut self, channel: usize) -> Self::ChannelMut<'_> {
        self.buf.channel_mut(channel).chunk(self.n, self.len)
    }

    fn copy_channels(&mut self, from: usize, to: usize)
    where
        Self::Sample: Copy,
    {
        self.buf.copy_channels(from, to);
    }
}

/// ```rust
/// use audio::{Buf, ExactSizeBuf};
///
/// let buf = audio::interleaved![[0; 4]; 2];
///
/// assert_eq!((&buf).chunk(0, 3).frames(), 3);
/// assert_eq!((&buf).chunk(1, 3).frames(), 1);
/// assert_eq!((&buf).chunk(2, 3).frames(), 0);
/// ```
impl<B> ExactSizeBuf for Chunk<B>
where
    B: ExactSizeBuf,
{
    fn frames(&self) -> usize {
        let len = self
            .buf
            .frames()
            .saturating_sub(self.n.saturating_mul(self.len));
        usize::min(len, self.len)
    }
}
