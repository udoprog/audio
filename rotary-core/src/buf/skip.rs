use crate::buf::{
    AsInterleaved, AsInterleavedMut, Buf, BufMut, ExactSizeBuf, InterleavedBuf, ResizableBuf,
};
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

    #[inline]
    fn calculate_frames(&self, frames: usize) -> usize {
        frames.saturating_add(self.n)
    }
}

impl<B> ExactSizeBuf for Skip<B>
where
    B: ExactSizeBuf,
{
    fn frames(&self) -> usize {
        self.buf.frames().saturating_sub(self.n)
    }
}

impl<B, T> Buf<T> for Skip<B>
where
    B: Buf<T>,
{
    fn frames_hint(&self) -> Option<usize> {
        let frames = self.buf.frames_hint()?;
        Some(frames.saturating_sub(self.n))
    }

    fn channels(&self) -> usize {
        self.buf.channels()
    }

    fn channel(&self, channel: usize) -> Channel<'_, T> {
        self.buf.channel(channel).skip(self.n)
    }
}

impl<B> ResizableBuf for Skip<B>
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

impl<B> InterleavedBuf for Skip<B>
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

impl<B, T> AsInterleaved<T> for Skip<B>
where
    B: AsInterleaved<T> + Buf<T>,
{
    fn as_interleaved(&self) -> &[T] {
        let channels = self.buf.channels();
        let buf = self.buf.as_interleaved();
        let start = usize::min(buf.len(), self.n.saturating_mul(channels));
        buf.get(start..).unwrap_or_default()
    }
}

impl<B, T> AsInterleavedMut<T> for Skip<B>
where
    B: AsInterleavedMut<T> + Buf<T>,
{
    fn as_interleaved_mut(&mut self) -> &mut [T] {
        let channels = self.buf.channels();
        let buf = self.buf.as_interleaved_mut();
        let start = usize::min(buf.len(), self.n.saturating_mul(channels));
        buf.get_mut(start..).unwrap_or_default()
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
