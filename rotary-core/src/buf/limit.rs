use crate::buf::{
    AsInterleaved, AsInterleavedMut, Buf, Channels, ChannelsMut, ExactSizeBuf, InterleavedBuf,
    ResizableBuf,
};
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

    #[inline]
    fn calculate_frames(&self, frames: usize) -> usize
    where
        B: ExactSizeBuf,
    {
        self.buf
            .frames()
            .saturating_sub(self.limit)
            .saturating_add(self.limit)
            .saturating_add(frames)
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

impl<B, T> Channels<T> for Limit<B>
where
    B: Channels<T>,
{
    fn channel(&self, channel: usize) -> Channel<'_, T> {
        self.buf.channel(channel).limit(self.limit)
    }
}

impl<B> ResizableBuf for Limit<B>
where
    B: ExactSizeBuf + ResizableBuf,
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

impl<B> InterleavedBuf for Limit<B>
where
    B: ExactSizeBuf + InterleavedBuf,
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

impl<B, T> AsInterleaved<T> for Limit<B>
where
    B: Buf + AsInterleaved<T>,
{
    fn as_interleaved(&self) -> &[T] {
        let channels = self.buf.channels();
        let buf = self.buf.as_interleaved();
        let end = usize::min(buf.len(), self.limit.saturating_mul(channels));
        buf.get(..end).unwrap_or_default()
    }
}

impl<B, T> AsInterleavedMut<T> for Limit<B>
where
    B: Buf + AsInterleavedMut<T>,
{
    fn as_interleaved_mut(&mut self) -> &mut [T] {
        let channels = self.buf.channels();
        let buf = self.buf.as_interleaved_mut();
        let end = usize::min(buf.len(), self.limit.saturating_mul(channels));
        buf.get_mut(..end).unwrap_or_default()
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
