use crate::buf::{Buf, BufInfo, BufMut, ReadBuf};
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

    /// Explicitly set the number of frames that has been written, it is assumed
    /// that we read from 0 when this is called.
    pub fn set_written(&mut self, written: usize) {
        self.read_at = 0;
        self.write_at = written;
    }

    /// Test if buffer has remaining data.
    pub fn has_remaining(&self) -> bool {
        self.remaining() > 0
    }

    /// The remaining number of frames available.
    pub fn remaining(&self) -> usize {
        self.write_at.saturating_sub(self.read_at)
    }

    /// Test if buffer has remaining data.
    pub fn has_remaining_mut(&self) -> bool {
        self.remaining_mut() > 0
    }

    /// The remaining number of frames available.
    pub fn remaining_mut(&self) -> usize {
        self.buf.buf_info_frames().saturating_sub(self.write_at)
    }

    /// Access the underlying buffer.
    pub fn read(&mut self) -> Read<'_, B> {
        Read {
            buf: &self.buf,
            end: self.write_at,
            read_at: &mut self.read_at,
        }
    }

    /// Write to the underlying buffer.
    pub fn copy<T, I>(&mut self, mut buf: I)
    where
        B: BufMut<T>,
        T: Sample,
        I: ReadBuf + Buf<T>,
    {
        let len = usize::min(self.remaining_mut(), buf.buf_info_frames());
        crate::utils::copy(&buf, (&mut self.buf).skip(self.write_at));
        self.write_at = self.write_at.saturating_add(len);
        buf.advance(len);
    }

    /// Write translated samples to the underlying buffer.
    pub fn translate<T, I, U>(&mut self, mut buf: I)
    where
        B: BufMut<T>,
        T: Sample + Translate<U>,
        I: ReadBuf + Buf<U>,
        U: Sample,
    {
        let len = usize::min(self.remaining_mut(), buf.buf_info_frames());
        crate::utils::translate(&buf, (&mut self.buf).skip(self.write_at));
        self.write_at = self.write_at.saturating_add(len);
        buf.advance(len);
    }
}

pub struct Read<'a, B> {
    buf: &'a B,
    end: usize,
    read_at: &'a mut usize,
}

impl<'a, B> BufInfo for Read<'a, B>
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

impl<'a, B, T> Buf<T> for Read<'a, B>
where
    B: Buf<T>,
    T: Sample,
{
    fn channel(&self, channel: usize) -> crate::Channel<'_, T> {
        let len = self.remaining();
        self.buf.channel(channel).skip(*self.read_at).limit(len)
    }
}

impl<'a, B> ReadBuf for Read<'a, B> {
    fn remaining(&self) -> usize {
        self.end.saturating_sub(*self.read_at)
    }

    fn advance(&mut self, n: usize) {
        *self.read_at = self.read_at.saturating_add(n);
    }
}
