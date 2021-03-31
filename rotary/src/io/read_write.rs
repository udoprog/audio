use rotary_core::{Buf, BufMut, Channel, ChannelMut, ExactSizeBuf, ReadBuf, WriteBuf};

/// Make any mutable buffer into a write adapter that implements
/// [ReadBuf] and [WriteBuf].
///
/// # Examples
///
/// ```rust
/// use rotary::{Buf as _, ReadBuf as _, WriteBuf as _};
/// use rotary::io;
///
/// let from = rotary::interleaved![[1.0f32, 2.0f32, 3.0f32, 4.0f32]; 2];
/// let to = rotary::interleaved![[0.0f32; 4]; 2];
///
/// // Make `to` into a read / write adapter.
/// let mut to = io::ReadWrite::new(to);
///
/// io::copy_remaining(io::Read::new((&from).skip(2).limit(1)), &mut to);
/// assert_eq!(to.remaining(), 1);
///
/// io::copy_remaining(io::Read::new((&from).limit(1)), &mut to);
/// assert_eq!(to.remaining(), 2);
///
/// assert_eq! {
///     to.as_ref().as_slice(),
///     &[3.0, 3.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0],
/// };
///
/// // Note: 4 channels, 2 frames each.
/// let mut read_out = io::Write::new(rotary::Interleaved::with_topology(4, 2));
///
/// assert_eq!(read_out.remaining_mut(), 2);
/// assert!(read_out.has_remaining_mut());
///
/// assert_eq!(to.remaining(), 2);
/// assert!(to.has_remaining());
///
/// io::copy_remaining(&mut to, &mut read_out);
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
    /// use rotary::Buf as _;
    /// use rotary::io;
    ///
    /// let buffer: rotary::Interleaved<i16> = rotary::interleaved![[1, 2, 3, 4]; 4];
    /// let mut buffer = io::ReadWrite::new(buffer);
    ///
    /// let from = rotary::wrap::interleaved(&[1i16, 2i16, 3i16, 4i16][..], 2);
    ///
    /// io::translate_remaining(from, &mut buffer);
    ///
    /// assert_eq!(buffer.as_ref().channels(), 4);
    /// ```
    #[inline]
    pub fn as_ref(&self) -> &B {
        &self.buf
    }

    /// Access the underlying buffer mutably.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::Buf as _;
    /// use rotary::io;
    ///
    /// let buffer: rotary::Interleaved<i16> = rotary::interleaved![[1, 2, 3, 4]; 4];
    /// let mut buffer = io::ReadWrite::new(buffer);
    ///
    /// let from = rotary::wrap::interleaved(&[1i16, 2i16, 3i16, 4i16][..], 2);
    ///
    /// io::translate_remaining(from, &mut buffer);
    ///
    /// buffer.as_mut().resize_channels(2);
    ///
    /// assert_eq!(buffer.channels(), 2);
    /// ```
    #[inline]
    pub fn as_mut(&mut self) -> &mut B {
        &mut self.buf
    }

    /// Convert into the underlying buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::Buf as _;
    /// use rotary::io;
    ///
    /// let buffer: rotary::Interleaved<i16> = rotary::interleaved![[1, 2, 3, 4]; 4];
    /// let mut buffer = io::ReadWrite::new(buffer);
    ///
    /// let from = rotary::wrap::interleaved(&[1i16, 2i16, 3i16, 4i16][..], 2);
    ///
    /// io::translate_remaining(from, &mut buffer);
    ///
    /// let buffer = buffer.into_inner();
    ///
    /// assert_eq!(buffer.channels(), 4);
    /// ```
    #[inline]
    pub fn into_inner(self) -> B {
        self.buf
    }

    /// Clear the state of the read / write adapter, setting both read and
    /// written to zero.
    #[inline]
    pub fn clear(&mut self) {
        self.read = 0;
        self.written = 0;
    }

    /// Set the number of frames which have been read.
    ///
    /// This is clamped to always be < written.
    #[inline]
    pub fn set_read(&mut self, read: usize) {
        self.read = read;
    }

    /// Set the number of frames which have been written.
    #[inline]
    pub fn set_written(&mut self, written: usize) {
        self.written = written;
    }
}

impl<B> ExactSizeBuf for ReadWrite<B>
where
    B: ExactSizeBuf,
{
    fn frames(&self) -> usize {
        self.buf.frames()
    }
}

impl<B, T> Buf<T> for ReadWrite<B>
where
    B: Buf<T>,
{
    #[inline]
    fn frames_hint(&self) -> Option<usize> {
        self.buf.frames_hint()
    }

    #[inline]
    fn channels(&self) -> usize {
        self.buf.channels()
    }

    #[inline]
    fn channel(&self, channel: usize) -> Channel<'_, T> {
        let len = self.remaining();
        self.buf.channel(channel).skip(self.read).limit(len)
    }
}

impl<B, T> BufMut<T> for ReadWrite<B>
where
    B: ExactSizeBuf + BufMut<T>,
{
    #[inline]
    fn channel_mut(&mut self, channel: usize) -> ChannelMut<'_, T> {
        self.buf.channel_mut(channel).skip(self.written)
    }
}

impl<B> ReadBuf for ReadWrite<B> {
    #[inline]
    fn remaining(&self) -> usize {
        self.written.saturating_sub(self.read)
    }

    #[inline]
    fn advance(&mut self, n: usize) {
        self.read = self.read.saturating_add(n);
    }
}

impl<B> WriteBuf for ReadWrite<B>
where
    B: ExactSizeBuf,
{
    #[inline]
    fn remaining_mut(&self) -> usize {
        self.buf.frames().saturating_sub(self.written)
    }

    #[inline]
    fn advance_mut(&mut self, n: usize) {
        self.written = self.written.saturating_add(n);
    }
}
