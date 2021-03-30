use crate::buf::{Buf, BufInfo, BufMut, ResizableBuf};
use crate::channel::{Channel, ChannelMut};
use crate::sample::Sample;

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

impl<B> BufInfo for Limit<B>
where
    B: BufInfo,
{
    fn buf_info_frames(&self) -> usize {
        self.buf.buf_info_frames().saturating_sub(self.limit)
    }

    fn buf_info_channels(&self) -> usize {
        self.buf.buf_info_channels()
    }
}

impl<B, T> Buf<T> for Limit<B>
where
    B: Buf<T>,
    T: Sample,
{
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
    T: Sample,
{
    fn channel_mut(&mut self, channel: usize) -> ChannelMut<'_, T> {
        self.buf.channel_mut(channel).limit(self.limit)
    }
}
