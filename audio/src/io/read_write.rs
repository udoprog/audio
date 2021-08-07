use audio_core::{Buf, Channel, Channels, ChannelsMut, ExactSizeBuf, ReadBuf, WriteBuf};

/// Make any mutable buffer into a write adapter that implements
/// [ReadBuf] and [WriteBuf].
///
/// # Examples
///
/// ```rust
/// use audio::{Buf as _, ReadBuf as _, WriteBuf as _};
/// use audio::io;
///
/// let from = audio::interleaved![[1.0f32, 2.0f32, 3.0f32, 4.0f32]; 2];
/// let to = audio::interleaved![[0.0f32; 4]; 2];
///
/// // Make `to` into a read / write adapter.
/// let mut to = io::ReadWrite::empty(to);
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
/// let mut read_out = io::Write::new(audio::Interleaved::with_topology(4, 2));
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
    /// Construct a new adapter that supports both reading and writing.
    ///
    /// The constructed reader will be initialized so that the number of bytes
    /// available for reading are equal to what's reported by
    /// [ExactSizeBuf::frames].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::{ReadBuf, ExactSizeBuf};
    /// use audio::io;
    ///
    /// let buffer = audio::interleaved![[1, 2, 3, 4], [5, 6, 7, 8]];
    /// assert_eq!(buffer.frames(), 4);
    ///
    /// let buffer = io::ReadWrite::new(buffer);
    ///
    /// assert!(buffer.has_remaining());
    /// assert_eq!(buffer.remaining(), 4);
    /// ```
    pub fn new(buf: B) -> Self
    where
        B: ExactSizeBuf,
    {
        let written = buf.frames();

        Self {
            buf,
            read: 0,
            written,
        }
    }

    /// Construct a new adapter that supports both reading and writing.
    ///
    /// The constructed reader will be initialized so that there have been no
    /// frames written to it, so there will not be any frames available for
    /// reading.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::{ReadBuf, ExactSizeBuf};
    /// use audio::io;
    ///
    /// let buffer = audio::interleaved![[1, 2, 3, 4], [5, 6, 7, 8]];
    /// assert_eq!(buffer.frames(), 4);
    ///
    /// let buffer = io::ReadWrite::empty(buffer);
    ///
    /// assert!(!buffer.has_remaining());
    /// assert_eq!(buffer.remaining(), 0);
    /// ```
    pub fn empty(buf: B) -> Self {
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
    /// use audio::Channels as _;
    /// use audio::io;
    ///
    /// let buffer: audio::Interleaved<i16> = audio::interleaved![[1, 2, 3, 4]; 4];
    /// let mut buffer = io::ReadWrite::new(buffer);
    ///
    /// let from = audio::wrap::interleaved(&[1i16, 2i16, 3i16, 4i16][..], 2);
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
    /// use audio::Buf as _;
    /// use audio::io;
    ///
    /// let to: audio::Interleaved<i16> = audio::interleaved![[1, 2, 3, 4]; 4];
    /// let mut to = io::ReadWrite::new(to);
    ///
    /// let from = audio::wrap::interleaved(&[1i16, 2i16, 3i16, 4i16][..], 2);
    ///
    /// io::translate_remaining(from, &mut to);
    ///
    /// to.as_mut().resize_channels(2);
    ///
    /// assert_eq!(to.channels(), 2);
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
    /// use audio::Channels as _;
    /// use audio::io;
    ///
    /// let buffer: audio::Interleaved<i16> = audio::interleaved![[1, 2, 3, 4]; 4];
    /// let mut buffer = io::ReadWrite::new(buffer);
    ///
    /// let from = audio::wrap::interleaved(&[1i16, 2i16, 3i16, 4i16][..], 2);
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

    /// Set the number of frames read.
    ///
    /// This can be used to rewind the internal cursor to a previously written
    /// frame if needed. Or, if the underlying buffer has changed for some
    /// reason, like if it was written to through a call to [ReadWrite::as_mut].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::{Buf, Channels, ReadBuf};
    /// use audio::io;
    ///
    /// fn read_from_buf(mut read: impl Channels<Sample = i16> + ReadBuf) {
    ///     let mut out = audio::interleaved![[0; 4]; 2];
    ///     io::copy_remaining(read, io::Write::new(&mut out));
    /// }
    ///
    /// let mut buffer = io::ReadWrite::new(audio::interleaved![[1, 2, 3, 4], [5, 6, 7, 8]]);
    /// read_from_buf(&mut buffer);
    ///
    /// assert!(!buffer.has_remaining());
    ///
    /// buffer.set_read(0);
    ///
    /// assert!(buffer.has_remaining());
    /// ```
    #[inline]
    pub fn set_read(&mut self, read: usize) {
        self.read = read;
    }

    /// Set the number of frames written.
    ///
    /// This can be used to rewind the internal cursor to a previously written
    /// frame if needed. Or, if the underlying buffer has changed for some
    /// reason, like if it was read into through a call to [ReadWrite::as_mut].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::{ChannelsMut, WriteBuf};
    /// use audio::io;
    ///
    /// fn write_to_buf(mut write: impl ChannelsMut<Sample = i16> + WriteBuf) {
    ///     let mut from = audio::interleaved![[0; 4]; 2];
    ///     io::copy_remaining(io::Read::new(&mut from), write);
    /// }
    ///
    /// let mut buffer = io::ReadWrite::new(audio::interleaved![[1, 2, 3, 4], [5, 6, 7, 8]]);
    /// write_to_buf(&mut buffer);
    ///
    /// assert!(!buffer.has_remaining_mut());
    ///
    /// buffer.set_written(0);
    ///
    /// assert!(buffer.has_remaining_mut());
    /// ```
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

impl<B> Buf for ReadWrite<B>
where
    B: Buf,
{
    #[inline]
    fn frames_hint(&self) -> Option<usize> {
        self.buf.frames_hint()
    }

    #[inline]
    fn channels(&self) -> usize {
        self.buf.channels()
    }
}

impl<B> Channels for ReadWrite<B>
where
    B: Channels,
{
    type Sample = B::Sample;

    type Channel<'a>
    where
        Self::Sample: 'a,
    = B::Channel<'a>;

    #[inline]
    fn channel(&self, channel: usize) -> Self::Channel<'_> {
        let len = self.remaining();
        self.buf.channel(channel).skip(self.read).limit(len)
    }
}

impl<B> ChannelsMut for ReadWrite<B>
where
    B: ExactSizeBuf + ChannelsMut,
{
    type ChannelMut<'a>
    where
        Self::Sample: 'a,
    = B::ChannelMut<'a>;

    #[inline]
    fn channel_mut(&mut self, channel: usize) -> Self::ChannelMut<'_> {
        self.buf.channel_mut(channel).skip(self.written)
    }

    #[inline]
    fn copy_channels(&mut self, from: usize, to: usize)
    where
        Self::Sample: Copy,
    {
        self.buf.copy_channels(from, to);
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
