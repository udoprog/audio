use crate::buf::{
    AsInterleaved, AsInterleavedMut, Buf, Channel, ChannelMut, Channels, ChannelsMut, ExactSizeBuf,
    InterleavedBuf, ResizableBuf,
};

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

    #[inline]
    fn calculate_frames(&self, frames: usize) -> usize {
        frames.saturating_add(self.n).saturating_mul(self.len)
    }
}

impl<B> ExactSizeBuf for Chunk<B>
where
    B: ExactSizeBuf,
{
    fn frames(&self) -> usize {
        self.buf.frames().saturating_sub(self.n * self.len)
    }
}

impl<B> Buf for Chunk<B>
where
    B: Buf,
{
    fn frames_hint(&self) -> Option<usize> {
        let frames = self.buf.frames_hint()?;
        Some(frames.saturating_sub(self.n * self.len))
    }

    fn channels(&self) -> usize {
        self.buf.channels()
    }
}

impl<B, T> Channels<T> for Chunk<B>
where
    B: Channels<T>,
{
    fn channel(&self, channel: usize) -> Channel<'_, T> {
        self.buf.channel(channel).chunk(self.n, self.len)
    }
}

impl<B> ResizableBuf for Chunk<B>
where
    B: ResizableBuf,
{
    fn resize(&mut self, frames: usize) {
        let frames = self.calculate_frames(frames);
        self.buf.resize(frames);
    }

    fn resize_topology(&mut self, channels: usize, frames: usize) {
        let frames = self.calculate_frames(frames);
        self.buf.resize_topology(channels, frames);
    }
}

impl<B> InterleavedBuf for Chunk<B>
where
    B: InterleavedBuf,
{
    fn reserve_frames(&mut self, frames: usize) {
        let frames = self.calculate_frames(frames);
        self.buf.reserve_frames(frames);
    }

    fn set_topology(&mut self, channels: usize, frames: usize) {
        let frames = self.calculate_frames(frames);
        self.buf.set_topology(channels, frames);
    }
}

impl<B, T> AsInterleaved<T> for Chunk<B>
where
    B: Buf + AsInterleaved<T>,
{
    fn as_interleaved(&self) -> &[T] {
        let channels = self.buf.channels();
        let len = self.len.saturating_mul(channels);
        let buf = self.buf.as_interleaved();
        let start = usize::min(self.n.saturating_mul(len), buf.len());
        let end = usize::min(start.saturating_add(len), buf.len());
        buf.get(start..end).unwrap_or_default()
    }
}

impl<B, T> AsInterleavedMut<T> for Chunk<B>
where
    B: Buf + AsInterleavedMut<T>,
{
    fn as_interleaved_mut(&mut self) -> &mut [T] {
        let channels = self.buf.channels();
        let len = self.len.saturating_mul(channels);
        let buf = self.buf.as_interleaved_mut();
        let start = usize::min(self.n.saturating_mul(len), buf.len());
        let end = usize::min(start.saturating_add(len), buf.len());
        buf.get_mut(start..end).unwrap_or_default()
    }
}

impl<B, T> ChannelsMut<T> for Chunk<B>
where
    B: ChannelsMut<T>,
{
    fn channel_mut(&mut self, channel: usize) -> ChannelMut<'_, T> {
        self.buf.channel_mut(channel).chunk(self.n, self.len)
    }

    fn copy_channels(&mut self, from: usize, to: usize)
    where
        T: Copy,
    {
        self.buf.copy_channels(from, to);
    }
}
