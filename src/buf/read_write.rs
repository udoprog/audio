use crate::buf::{Buf, BufInfo, BufMut};
use crate::buf_io::{ReadBuf, WriteBuf};
use crate::sample::Sample;
use crate::translate::Translate;

/// An abstraction intended for both reading and writing to buffer.
pub struct ReadWrite<B> {
    buf: B,
    // Number of bytes available for reading. Conversely, the number of bytes
    // available for writing is the length of the buffer subtracted by this.
    read_at: usize,
    // The position in frames to write at.
    write_at: usize,
}

impl<B> ReadWrite<B>
where
    B: BufInfo,
{
    pub(super) fn new(buf: B) -> Self {
        Self {
            buf,
            read_at: 0,
            write_at: 0,
        }
    }

    /// Access the underlying buffer.
    pub fn as_ref(&self) -> &B {
        &self.buf
    }

    /// Access the underlying buffer mutably.
    pub fn as_mut(&mut self) -> &mut B {
        &mut self.buf
    }

    /// Explicitly clear the number of frames read and written to allow for
    /// re-using the underlying buffer, assuming it's been fully consumed.
    pub fn clear(&mut self) {
        self.read_at = 0;
        self.write_at = 0;
    }

    /// Explicitly set the number of frames that has been written, it is assumed
    /// that we read from 0 when this is called.
    pub fn set_written(&mut self, written: usize) {
        self.read_at = 0;
        self.write_at = written;
    }
}

impl<B> BufInfo for ReadWrite<B>
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

impl<B, T> Buf<T> for ReadWrite<B>
where
    B: Buf<T>,
    T: Sample,
{
    fn channel(&self, channel: usize) -> crate::Channel<'_, T> {
        let len = self.remaining();
        self.buf.channel(channel).skip(self.read_at).limit(len)
    }
}

impl<B> ReadBuf for ReadWrite<B> {
    fn remaining(&self) -> usize {
        self.write_at.saturating_sub(self.read_at)
    }

    fn advance(&mut self, n: usize) {
        self.read_at = self.read_at.saturating_add(n);
    }
}

impl<B, T> WriteBuf<T> for ReadWrite<B>
where
    B: BufMut<T>,
    T: Sample,
{
    fn remaining_mut(&self) -> usize {
        self.buf.buf_info_frames().saturating_sub(self.write_at)
    }

    fn copy<I>(&mut self, mut buf: I)
    where
        I: ReadBuf + Buf<T>,
    {
        let len = usize::min(self.remaining_mut(), buf.remaining());
        crate::utils::copy(&buf, (&mut self.buf).skip(self.write_at));
        self.write_at = self.write_at.saturating_add(len);
        buf.advance(len);
    }

    fn translate<I, U>(&mut self, mut buf: I)
    where
        T: Translate<U>,
        I: ReadBuf + Buf<U>,
        U: Sample,
    {
        let len = usize::min(self.remaining_mut(), buf.remaining());
        crate::utils::translate(&buf, (&mut self.buf).skip(self.write_at));
        self.write_at = self.write_at.saturating_add(len);
        buf.advance(len);
    }
}
