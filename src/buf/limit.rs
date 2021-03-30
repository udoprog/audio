use crate::buf::{Buf, BufMut, ChannelSlice, ChannelSliceKind, ChannelSliceMut};
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

    fn channel(&self, channel: usize) -> ChannelSlice<'_, T> {
        let ChannelSlice { buf, kind } = self.buf.channel(channel);

        match kind {
            ChannelSliceKind::Linear => ChannelSlice {
                buf: &buf[..self.limit],
                kind,
            },
            ChannelSliceKind::Interleaved { channels, .. } => ChannelSlice {
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

    fn channel_mut(&mut self, channel: usize) -> ChannelSliceMut<'_, T> {
        let ChannelSliceMut { buf, kind } = self.buf.channel_mut(channel);

        match kind {
            ChannelSliceKind::Linear => ChannelSliceMut {
                buf: &mut buf[..self.limit],
                kind,
            },
            ChannelSliceKind::Interleaved { channels, .. } => ChannelSliceMut {
                buf: &mut buf[..self.limit * channels],
                kind,
            },
        }
    }
}
