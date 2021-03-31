use crate::buf::{Buf, BufInfo, BufMut, ResizableBuf};
use crate::channel::{Channel, ChannelMut};
use crate::io::ReadBuf;

/// A buffer where a number of frames have been skipped over.
///
/// Created with [Buf::skip].
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

impl<B> BufInfo for Skip<B>
where
    B: BufInfo,
{
    fn buf_info_frames(&self) -> usize {
        self.buf.buf_info_frames().saturating_sub(self.n)
    }

    fn buf_info_channels(&self) -> usize {
        self.buf.buf_info_channels()
    }
}

impl<B, T> Buf<T> for Skip<B>
where
    B: Buf<T>,
{
    fn channel(&self, channel: usize) -> Channel<'_, T> {
        self.buf.channel(channel).skip(self.n)
    }
}

impl<B> ResizableBuf for Skip<B>
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

impl<B, T> BufMut<T> for Skip<B>
where
    B: BufMut<T>,
{
    fn channel_mut(&mut self, channel: usize) -> ChannelMut<'_, T> {
        self.buf.channel_mut(channel).skip(self.n)
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
