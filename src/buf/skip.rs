use crate::buf::{Buf, BufMut, Channel, ChannelMut};
use crate::sample::Sample;

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

impl<B, T> Buf<T> for Skip<B>
where
    B: Buf<T>,
    T: Sample,
{
    fn frames(&self) -> usize {
        self.buf.frames() - self.n
    }

    fn channels(&self) -> usize {
        self.buf.channels()
    }

    fn channel(&self, channel: usize) -> Channel<'_, T> {
        self.buf.channel(channel).skip(self.n)
    }
}

impl<B, T> BufMut<T> for Skip<B>
where
    B: BufMut<T>,
    T: Sample,
{
    fn resize(&mut self, frames: usize) {
        self.buf.resize(frames + self.n);
    }

    fn resize_topology(&mut self, channels: usize, frames: usize) {
        self.buf.resize_topology(channels, frames + self.n);
    }

    fn channel_mut(&mut self, channel: usize) -> ChannelMut<'_, T> {
        self.buf.channel_mut(channel).skip(self.n)
    }
}
