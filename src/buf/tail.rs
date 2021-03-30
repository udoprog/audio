use crate::buf::{Buf, BufMut, Channel, ChannelMut};
use crate::sample::Sample;

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

impl<B, T> Buf<T> for Tail<B>
where
    B: Buf<T>,
    T: Sample,
{
    fn frames(&self) -> usize {
        usize::min(self.buf.frames(), self.n)
    }

    fn channels(&self) -> usize {
        self.buf.channels()
    }

    fn channel(&self, channel: usize) -> Channel<'_, T> {
        self.buf.channel(channel).tail(self.n)
    }
}

impl<B, T> BufMut<T> for Tail<B>
where
    B: BufMut<T>,
    T: Sample,
{
    fn resize(&mut self, frames: usize) {
        self.buf.resize(frames.saturating_add(self.n));
    }

    fn resize_topology(&mut self, channels: usize, frames: usize) {
        self.buf
            .resize_topology(channels, frames.saturating_add(self.n));
    }

    fn channel_mut(&mut self, channel: usize) -> ChannelMut<'_, T> {
        self.buf.channel_mut(channel).tail(self.n)
    }
}
