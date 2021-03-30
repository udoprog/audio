use crate::buf::{Buf, BufInfo};
use crate::buf_io::ReadBuf;
use crate::channel::Channel;
use crate::sample::Sample;

/// A reader abstraction allowing a caller to keep track of how many frames are
/// remaining to read.
pub struct Read<B> {
    buf: B,
    available: usize,
}

impl<B> Read<B>
where
    B: BufInfo,
{
    pub(super) fn new(buf: B) -> Self {
        let available = buf.buf_info_frames();
        Self { buf, available }
    }

    /// Access the underlying buffer immutably.
    pub fn as_ref(&self) -> &B {
        &self.buf
    }

    /// Access the underlying buffer mutably.
    pub fn as_mut(&mut self) -> &mut B {
        &mut self.buf
    }
}

impl<B> ReadBuf for Read<B> {
    fn remaining(&self) -> usize {
        self.available
    }

    fn advance(&mut self, n: usize) {
        self.available = self.available.saturating_sub(n);
    }
}

impl<B> BufInfo for Read<B>
where
    B: BufInfo,
{
    fn buf_info_frames(&self) -> usize {
        self.buf.buf_info_frames()
    }

    fn buf_info_channels(&self) -> usize {
        self.buf.buf_info_channels()
    }
}

impl<B, T> Buf<T> for Read<B>
where
    B: Buf<T>,
    T: Sample,
{
    fn channel(&self, channel: usize) -> Channel<'_, T> {
        self.buf.channel(channel).tail(self.available)
    }
}
