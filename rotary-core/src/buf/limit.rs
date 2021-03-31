use crate::buf::{Buf, BufMut, ExactSizeBuf, ResizableBuf};
use crate::channel::{Channel, ChannelMut};
use crate::io::ReadBuf;

/// A buffer that has been limited.
///
/// Created with [Buf::limit].
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

impl<B> ExactSizeBuf for Limit<B>
where
    B: ExactSizeBuf,
{
    fn frames(&self) -> usize {
        usize::min(self.buf.frames(), self.limit)
    }
}

impl<B, T> Buf<T> for Limit<B>
where
    B: Buf<T>,
{
    fn frames_hint(&self) -> Option<usize> {
        let frames = self.buf.frames_hint()?;
        Some(usize::min(frames, self.limit))
    }

    fn channels(&self) -> usize {
        self.buf.channels()
    }

    fn channel(&self, channel: usize) -> Channel<'_, T> {
        self.buf.channel(channel).limit(self.limit)
    }
}

impl<B> ResizableBuf for Limit<B>
where
    B: ResizableBuf,
{
    fn resize(&mut self, frames: usize) {
        self.buf.resize(frames.saturating_add(self.limit));
    }

    fn resize_topology(&mut self, channels: usize, frames: usize) {
        self.buf
            .resize_topology(channels, frames.saturating_add(self.limit));
    }
}

impl<B, T> BufMut<T> for Limit<B>
where
    B: BufMut<T>,
{
    fn channel_mut(&mut self, channel: usize) -> ChannelMut<'_, T> {
        self.buf.channel_mut(channel).limit(self.limit)
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
