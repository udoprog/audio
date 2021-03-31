use crate::buf::{Buf, BufInfo, BufMut, ResizableBuf};
use crate::channel::{Channel, ChannelMut};
use crate::io::ReadBuf;

/// The tail of a buffer.
///
/// Created with [Buf::tail].
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

impl<B> BufInfo for Tail<B>
where
    B: BufInfo,
{
    fn frames(&self) -> usize {
        usize::min(self.buf.frames(), self.n)
    }

    fn channels(&self) -> usize {
        self.buf.channels()
    }
}

impl<B, T> Buf<T> for Tail<B>
where
    B: Buf<T>,
{
    fn channel(&self, channel: usize) -> Channel<'_, T> {
        self.buf.channel(channel).tail(self.n)
    }
}

impl<B> ResizableBuf for Tail<B>
where
    B: ResizableBuf,
{
    fn resize(&mut self, frames: usize) {
        self.buf.resize(frames.saturating_add(self.n));
    }

    fn resize_topology(&mut self, channels: usize, frames: usize) {
        self.buf
            .resize_topology(channels, frames.saturating_add(self.n));
    }
}

impl<B, T> BufMut<T> for Tail<B>
where
    B: BufMut<T>,
{
    fn channel_mut(&mut self, channel: usize) -> ChannelMut<'_, T> {
        self.buf.channel_mut(channel).tail(self.n)
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
