use crate::buf::{Buf, BufChannel, BufChannelKind, BufChannelMut, BufMut};

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
{
    fn frames(&self) -> usize {
        self.buf.frames() - self.limit
    }

    fn channels(&self) -> usize {
        self.buf.channels()
    }

    fn channel(&self, channel: usize) -> BufChannel<'_, T> {
        let BufChannel { buf, kind } = self.buf.channel(channel);

        match kind {
            BufChannelKind::Linear => BufChannel {
                buf: &buf[..self.limit],
                kind,
            },
            BufChannelKind::Interleaved { channels, .. } => BufChannel {
                buf: &buf[..self.limit * channels],
                kind,
            },
        }
    }
}

impl<B, T> BufMut<T> for Limit<B>
where
    B: BufMut<T>,
{
    fn resize(&mut self, frames: usize) {
        self.buf.resize(frames + self.limit);
    }

    fn resize_topology(&mut self, channels: usize, frames: usize) {
        self.buf.resize_topology(channels, frames + self.limit);
    }

    fn channel_mut(&mut self, channel: usize) -> BufChannelMut<'_, T> {
        let BufChannelMut { buf, kind } = self.buf.channel_mut(channel);

        match kind {
            BufChannelKind::Linear => BufChannelMut {
                buf: &mut buf[..self.limit],
                kind,
            },
            BufChannelKind::Interleaved { channels, .. } => BufChannelMut {
                buf: &mut buf[..self.limit * channels],
                kind,
            },
        }
    }
}
