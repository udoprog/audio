use crate::channel::{Channel, ChannelMut};
use crate::io::ReadBuf;
use crate::{
    buf::{AsInterleavedMut, Buf, BufMut, ExactSizeBuf, InterleavedBuf, ResizableBuf},
    AsInterleaved,
};

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

    #[inline]
    fn calculate_frames(&self, frames: usize) -> usize {
        frames.saturating_add(self.n)
    }
}

impl<B> ExactSizeBuf for Tail<B>
where
    B: ExactSizeBuf,
{
    fn frames(&self) -> usize {
        usize::min(self.buf.frames(), self.n)
    }
}

impl<B, T> Buf<T> for Tail<B>
where
    B: Buf<T>,
{
    fn frames_hint(&self) -> Option<usize> {
        let frames = self.buf.frames_hint()?;
        Some(usize::min(frames, self.n))
    }

    fn channels(&self) -> usize {
        self.buf.channels()
    }

    fn channel(&self, channel: usize) -> Channel<'_, T> {
        self.buf.channel(channel).tail(self.n)
    }
}

impl<B> ResizableBuf for Tail<B>
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

impl<B> InterleavedBuf for Tail<B>
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

impl<B, T> AsInterleaved<T> for Tail<B>
where
    B: AsInterleaved<T> + Buf<T>,
{
    fn as_interleaved(&self) -> &[T] {
        let channels = self.buf.channels();
        let buf = self.buf.as_interleaved();
        let tail = buf.len().saturating_sub(self.n.saturating_mul(channels));
        buf.get(tail..).unwrap_or_default()
    }
}

impl<B, T> AsInterleavedMut<T> for Tail<B>
where
    B: AsInterleavedMut<T> + Buf<T>,
{
    fn as_interleaved_mut(&mut self) -> &mut [T] {
        let channels = self.buf.channels();
        let buf = self.buf.as_interleaved_mut();
        let tail = buf.len().saturating_sub(self.n.saturating_mul(channels));
        buf.get_mut(tail..).unwrap_or_default()
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
