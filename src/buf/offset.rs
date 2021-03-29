use crate::buf::{Buf, BufChannel, BufChannelKind, BufChannelMut, BufMut};

/// A buffer that has been offset from the start.
///
/// Created with [Buf::offset].
pub struct Offset<B> {
    buf: B,
    offset: usize,
}

impl<B> Offset<B> {
    /// Construct a new buf offset.
    pub(crate) fn new(buf: B, offset: usize) -> Self {
        Self { buf, offset }
    }
}

impl<B, T> Buf<T> for Offset<B>
where
    B: Buf<T>,
{
    fn frames(&self) -> usize {
        self.buf.frames() - self.offset
    }

    fn channels(&self) -> usize {
        self.buf.channels()
    }

    fn channel(&self, channel: usize) -> BufChannel<'_, T> {
        let BufChannel { buf, kind } = self.buf.channel(channel);

        match kind {
            BufChannelKind::Linear => BufChannel {
                buf: &buf[self.offset..],
                kind,
            },
            BufChannelKind::Interleaved { channels, .. } => BufChannel {
                buf: &buf[self.offset * channels..],
                kind,
            },
        }
    }
}

impl<B, T> BufMut<T> for Offset<B>
where
    B: BufMut<T>,
{
    fn resize(&mut self, frames: usize) {
        self.buf.resize(frames + self.offset);
    }

    fn resize_topology(&mut self, channels: usize, frames: usize) {
        self.buf.resize_topology(channels, frames + self.offset);
    }

    fn channel_mut(&mut self, channel: usize) -> BufChannelMut<'_, T> {
        let BufChannelMut { buf, kind } = self.buf.channel_mut(channel);

        match kind {
            BufChannelKind::Linear => BufChannelMut {
                buf: &mut buf[self.offset..],
                kind,
            },
            BufChannelKind::Interleaved { channels, .. } => BufChannelMut {
                buf: &mut buf[self.offset * channels..],
                kind,
            },
        }
    }
}
