use crate::buf::{Buf, BufInfo, BufMut};
use crate::io::{ReadBuf, WriteBuf};
use crate::translate::Translate;

/// Make any mutable buffer into a write adapter that implements
/// [ReadBuf] and [WriteBuf].
///
/// # Examples
///
/// ```rust
/// use rotary::io::{Read, ReadWrite, Write};
/// use rotary::{Buf as _, ReadBuf as _, WriteBuf as _};
///
/// let from = rotary::interleaved![[1.0f32, 2.0f32, 3.0f32, 4.0f32]; 2];
/// let to = rotary::interleaved![[0.0f32; 4]; 2];
///
/// // Make `to` into a ReadWrite adapter.
/// let mut to = ReadWrite::new(to);
///
/// to.copy(Read::new((&from).skip(2).limit(1)));
/// assert_eq!(to.remaining(), 1);
///
/// to.copy(Read::new((&from).limit(1)));
/// assert_eq!(to.remaining(), 2);
///
/// assert_eq! {
///     to.as_ref().as_slice(),
///     &[3.0, 3.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0],
/// };
///
/// // Note: 4 channels, 2 frames each.
/// let mut read_out = Write::new(rotary::Interleaved::with_topology(4, 2));
///
/// assert_eq!(read_out.remaining_mut(), 2);
/// assert!(read_out.has_remaining_mut());
///
/// assert_eq!(to.remaining(), 2);
/// assert!(to.has_remaining());
///
/// read_out.copy(&mut to);
///
/// assert_eq!(read_out.remaining_mut(), 0);
/// assert!(!read_out.has_remaining_mut());
///
/// assert_eq!(to.remaining(), 0);
/// assert!(!to.has_remaining());
///
/// assert_eq! {
///     read_out.as_ref().as_slice(),
///     &[3.0, 3.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0],
/// }
/// ```
pub struct ReadWrite<B> {
    buf: B,
    // Number of bytes available for reading. Conversely, the number of bytes
    // available for writing is the length of the buffer subtracted by this.
    read: usize,
    // The position in frames to write at.
    written: usize,
}

impl<B> ReadWrite<B> {
    /// Construct a new read / write buffer around an audio buffer.
    pub fn new(buf: B) -> Self {
        Self {
            buf,
            read: 0,
            written: 0,
        }
    }

    /// Access the underlying buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::io::ReadWrite;
    /// use rotary::WriteBuf as _;
    ///
    /// let buffer: rotary::Interleaved<i16> = rotary::interleaved![[1, 2, 3, 4]; 4];
    /// let mut buffer = ReadWrite::new(buffer);
    ///
    /// let from = rotary::wrap::interleaved(&[1i16, 2i16, 3i16, 4i16][..], 2);
    ///
    /// buffer.translate(from);
    ///
    /// assert_eq!(buffer.as_ref().channels(), 4);
    /// ```
    pub fn as_ref(&self) -> &B {
        &self.buf
    }

    /// Access the underlying buffer mutably.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::io::ReadWrite;
    /// use rotary::{Buf as _, WriteBuf as _, BufInfo as _};
    ///
    /// let buffer: rotary::Interleaved<i16> = rotary::interleaved![[1, 2, 3, 4]; 4];
    /// let mut buffer = ReadWrite::new(buffer);
    ///
    /// let from = rotary::wrap::interleaved(&[1i16, 2i16, 3i16, 4i16][..], 2);
    ///
    /// buffer.translate(from);
    ///
    /// buffer.as_mut().resize_channels(2);
    ///
    /// assert_eq!(buffer.channels(), 2);
    /// ```
    pub fn as_mut(&mut self) -> &mut B {
        &mut self.buf
    }

    /// Convert into the underlying buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::io::ReadWrite;
    /// use rotary::WriteBuf as _;
    ///
    /// let buffer: rotary::Interleaved<i16> = rotary::interleaved![[1, 2, 3, 4]; 4];
    /// let mut buffer = ReadWrite::new(buffer);
    ///
    /// let from = rotary::wrap::interleaved(&[1i16, 2i16, 3i16, 4i16][..], 2);
    ///
    /// buffer.translate(from);
    ///
    /// let buffer = buffer.into_inner();
    ///
    /// assert_eq!(buffer.channels(), 4);
    /// ```
    pub fn into_inner(self) -> B {
        self.buf
    }

    /// Clear the state of the read / write adapter, setting both read and
    /// written to zero.
    pub fn clear(&mut self) {
        self.read = 0;
        self.written = 0;
    }

    /// Set the number of frames which have been read.
    ///
    /// This is clamped to always be < written.
    pub fn set_read(&mut self, read: usize) {
        self.read = usize::min(read, self.written);
    }

    /// Set the number of frames which have been written.
    pub fn set_written(&mut self, written: usize) {
        self.written = written;
    }
}

impl<B> BufInfo for ReadWrite<B>
where
    B: BufInfo,
{
    fn frames(&self) -> usize {
        self.buf.frames()
    }

    fn channels(&self) -> usize {
        self.buf.channels()
    }
}

impl<B, T> Buf<T> for ReadWrite<B>
where
    B: Buf<T>,
{
    fn channel(&self, channel: usize) -> crate::Channel<'_, T> {
        let len = self.remaining();
        self.buf.channel(channel).skip(self.read).limit(len)
    }
}

impl<B> ReadBuf for ReadWrite<B> {
    fn remaining(&self) -> usize {
        self.written.saturating_sub(self.read)
    }

    fn advance(&mut self, n: usize) {
        self.read = self.read.saturating_add(n);
    }
}

impl<B, T> WriteBuf<T> for ReadWrite<B>
where
    B: BufMut<T>,
{
    fn remaining_mut(&self) -> usize {
        self.buf.frames().saturating_sub(self.written)
    }

    fn copy<I>(&mut self, mut buf: I)
    where
        I: ReadBuf + Buf<T>,
        T: Copy,
    {
        let len = usize::min(self.remaining_mut(), buf.remaining());
        crate::io::utils::copy(&buf, (&mut self.buf).skip(self.written));
        self.written = self.written.saturating_add(len);
        buf.advance(len);
    }

    fn translate<I, U>(&mut self, mut buf: I)
    where
        T: Translate<U>,
        I: ReadBuf + Buf<U>,
        U: Copy,
    {
        let len = usize::min(self.remaining_mut(), buf.remaining());
        crate::io::utils::translate(&buf, (&mut self.buf).skip(self.written));
        self.written = self.written.saturating_add(len);
        buf.advance(len);
    }
}
