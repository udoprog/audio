use crate::buf::{Buf, BufMut, Channel, ChannelKind, ChannelMut};
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

impl<B, T> Buf<T> for Limit<B>
where
    B: Buf<T>,
    T: Sample,
{
    fn frames(&self) -> usize {
        self.buf.frames() - self.limit
    }

    fn channels(&self) -> usize {
        self.buf.channels()
    }

    fn channel(&self, channel: usize) -> Channel<'_, T> {
        let Channel { buf, kind } = self.buf.channel(channel);

        match kind {
            ChannelKind::Linear => Channel {
                buf: &buf[..self.limit],
                kind,
            },
            ChannelKind::Interleaved { channels, .. } => Channel {
                buf: &buf[..self.limit * channels],
                kind,
            },
        }
    }
}

impl<B, T> BufMut<T> for Limit<B>
where
    B: BufMut<T>,
    T: Sample,
{
    fn resize(&mut self, frames: usize) {
        self.buf.resize(frames + self.limit);
    }

    fn resize_topology(&mut self, channels: usize, frames: usize) {
        self.buf.resize_topology(channels, frames + self.limit);
    }

    fn channel_mut(&mut self, channel: usize) -> ChannelMut<'_, T> {
        let ChannelMut { buf, kind } = self.buf.channel_mut(channel);

        match kind {
            ChannelKind::Linear => ChannelMut {
                buf: &mut buf[..self.limit],
                kind,
            },
            ChannelKind::Interleaved { channels, .. } => ChannelMut {
                buf: &mut buf[..self.limit * channels],
                kind,
            },
        }
    }
}
