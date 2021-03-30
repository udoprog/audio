use crate::buf::{Buf, BufMut, Channel, ChannelKind, ChannelMut};
use crate::sample::Sample;

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
    T: Sample,
{
    fn frames(&self) -> usize {
        self.buf.frames() - self.offset
    }

    fn channels(&self) -> usize {
        self.buf.channels()
    }

    fn channel(&self, channel: usize) -> Channel<'_, T> {
        let Channel { buf, kind } = self.buf.channel(channel);

        match kind {
            ChannelKind::Linear => Channel {
                buf: &buf[self.offset..],
                kind,
            },
            ChannelKind::Interleaved { channels, .. } => Channel {
                buf: &buf[self.offset * channels..],
                kind,
            },
        }
    }
}

impl<B, T> BufMut<T> for Offset<B>
where
    B: BufMut<T>,
    T: Sample,
{
    fn resize(&mut self, frames: usize) {
        self.buf.resize(frames + self.offset);
    }

    fn resize_topology(&mut self, channels: usize, frames: usize) {
        self.buf.resize_topology(channels, frames + self.offset);
    }

    fn channel_mut(&mut self, channel: usize) -> ChannelMut<'_, T> {
        let ChannelMut { buf, kind } = self.buf.channel_mut(channel);

        match kind {
            ChannelKind::Linear => ChannelMut {
                buf: &mut buf[self.offset..],
                kind,
            },
            ChannelKind::Interleaved { channels, .. } => ChannelMut {
                buf: &mut buf[self.offset * channels..],
                kind,
            },
        }
    }
}
