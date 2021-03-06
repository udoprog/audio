use audio_core::{Buf, Channel, Channels, ExactSizeBuf, ReadBuf};

/// Make a buffer into a read adapter that implements [ReadBuf].
///
/// # Examples
///
/// ```rust
/// use audio::Buf;
/// use audio::io;
///
/// let from = audio::interleaved![[1, 2, 3, 4]; 2];
/// let mut to = audio::interleaved![[0; 4]; 2];
///
/// let mut to = io::ReadWrite::empty(to);
///
/// io::copy_remaining(io::Read::new((&from).skip(2).limit(1)), &mut to);
/// io::copy_remaining(io::Read::new((&from).limit(1)), &mut to);
///
/// assert_eq!(to.as_ref().as_slice(), &[3, 3, 1, 1, 0, 0, 0, 0]);
/// ```
pub struct Read<B> {
    buf: B,
    available: usize,
}

impl<B> Read<B> {
    /// Construct a new reading adapter.
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
    /// let buffer = io::Read::new(buffer);
    ///
    /// assert!(buffer.has_remaining());
    /// assert_eq!(buffer.remaining(), 4);
    /// ```
    #[inline]
    pub fn new(buf: B) -> Self
    where
        B: ExactSizeBuf,
    {
        let available = buf.frames();
        Self { buf, available }
    }

    /// Construct a new reading adapter.
    ///
    /// The constructed reader will be initialized so that there are no frames
    /// available for reading.
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
    /// let buffer = io::Read::empty(buffer);
    ///
    /// assert!(!buffer.has_remaining());
    /// assert_eq!(buffer.remaining(), 0);
    /// ```
    pub fn empty(buf: B) -> Self {
        Self { buf, available: 0 }
    }

    /// Access the underlying buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::Buf;
    /// use audio::{io, wrap};
    ///
    /// let from: audio::Interleaved<i16> = audio::interleaved![[1, 2, 3, 4]; 4];
    /// let mut from = io::Read::new(from);
    ///
    /// io::copy_remaining(&mut from, wrap::interleaved(&mut [0i16; 16][..], 4));
    ///
    /// assert_eq!(from.as_ref().channels(), 4);
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
    /// use audio::Buf;
    /// use audio::{io, wrap};
    ///
    /// let from: audio::Interleaved<i16> = audio::interleaved![[1, 2, 3, 4]; 4];
    /// let mut from = io::Read::new(from);
    ///
    /// io::copy_remaining(&mut from, wrap::interleaved(&mut [0i16; 16][..], 4));
    ///
    /// from.as_mut().resize_channels(2);
    ///
    /// assert_eq!(from.channels(), 2);
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
    /// use audio::Buf;
    /// use audio::{io, wrap};
    ///
    /// let from: audio::Interleaved<i16> = audio::interleaved![[1, 2, 3, 4]; 4];
    /// let mut from = io::Read::new(from);
    ///
    /// io::copy_remaining(&mut from, wrap::interleaved(&mut [0i16; 16][..], 4));
    ///
    /// let from = from.into_inner();
    ///
    /// assert_eq!(from.channels(), 4);
    /// ```
    #[inline]
    pub fn into_inner(self) -> B {
        self.buf
    }

    /// Set the number of frames read.
    ///
    /// This can be used to rewind the internal cursor to a previously written
    /// frame if needed. Or, if the underlying buffer has changed for some
    /// reason, like if it was written to through a call to [Read::as_mut].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::{Buf, Channels, ReadBuf};
    /// use audio::io;
    ///
    /// fn read_from_buf(mut read: impl Buf + Channels<i16> + ReadBuf) {
    ///     let mut out = audio::interleaved![[0; 4]; 2];
    ///     io::copy_remaining(read, io::Write::new(&mut out));
    /// }
    ///
    /// let mut buffer = io::Read::new(audio::interleaved![[1, 2, 3, 4], [5, 6, 7, 8]]);
    /// read_from_buf(&mut buffer);
    ///
    /// assert!(!buffer.has_remaining());
    ///
    /// buffer.set_read(0);
    ///
    /// assert!(buffer.has_remaining());
    /// ```
    #[inline]
    pub fn set_read(&mut self, read: usize)
    where
        B: ExactSizeBuf,
    {
        self.available = self.buf.frames().saturating_sub(read);
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

impl<B> ExactSizeBuf for Read<B>
where
    B: ExactSizeBuf,
{
    fn frames(&self) -> usize {
        self.buf.frames().saturating_sub(self.available)
    }
}

impl<B> Buf for Read<B>
where
    B: Buf,
{
    fn frames_hint(&self) -> Option<usize> {
        self.buf.frames_hint()
    }

    fn channels(&self) -> usize {
        self.buf.channels()
    }
}

impl<B, T> Channels<T> for Read<B>
where
    B: Channels<T>,
{
    fn channel(&self, channel: usize) -> Channel<'_, T> {
        self.buf.channel(channel).tail(self.available)
    }
}
