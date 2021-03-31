use rotary_core::Channel;
use rotary_core::ReadBuf;
use rotary_core::{Buf, ExactSizeBuf};

/// Make a buffer into a read adapter that implements [ReadBuf].
///
/// # Examples
///
/// ```rust
/// use rotary::{Buf as _, BufMut as _, WriteBuf as _};
/// use rotary::io::{Read, ReadWrite};
///
/// let from = rotary::interleaved![[1.0f32, 2.0f32, 3.0f32, 4.0f32]; 2];
/// let mut to = rotary::interleaved![[0.0f32; 4]; 2];
///
/// let mut to = ReadWrite::new(to);
///
/// to.copy(Read::new((&from).skip(2).limit(1)));
/// to.copy(Read::new((&from).limit(1)));
///
/// assert_eq! {
///     to.as_ref().as_slice(),
///     &[3.0, 3.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0],
/// };
/// ```
pub struct Read<B> {
    buf: B,
    available: usize,
}

impl<B> Read<B>
where
    B: ExactSizeBuf,
{
    /// Construct a new read adapter.
    pub fn new(buf: B) -> Self {
        let available = buf.frames();
        Self { buf, available }
    }

    /// Access the underlying buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::io::Read;
    /// use rotary::{Buf as _, WriteBuf as _};
    ///
    /// let buffer: rotary::Interleaved<i16> = rotary::interleaved![[1, 2, 3, 4]; 4];
    /// let mut buffer = Read::new(buffer);
    ///
    /// rotary::wrap::interleaved(&mut [0i16; 16][..], 4).copy(&mut buffer);
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
    /// use rotary::io::Read;
    /// use rotary::{Buf as _, WriteBuf as _, ExactSizeBuf as _};
    ///
    /// let buffer: rotary::Interleaved<i16> = rotary::interleaved![[1, 2, 3, 4]; 4];
    /// let mut buffer = Read::new(buffer);
    ///
    /// rotary::wrap::interleaved(&mut [0i16; 16][..], 4).copy(&mut buffer);
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
    /// use rotary::io::Read;
    /// use rotary::{Buf as _, WriteBuf as _};
    ///
    /// let buffer: rotary::Interleaved<i16> = rotary::interleaved![[1, 2, 3, 4]; 4];
    /// let mut buffer = Read::new(buffer);
    ///
    /// rotary::wrap::interleaved(&mut [0i16; 16][..], 4).copy(&mut buffer);
    ///
    /// let buffer = buffer.into_inner();
    ///
    /// assert_eq!(buffer.channels(), 4);
    /// ```
    pub fn into_inner(self) -> B {
        self.buf
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
        self.buf.frames()
    }
}

impl<B, T> Buf<T> for Read<B>
where
    B: Buf<T>,
{
    fn frames_hint(&self) -> Option<usize> {
        self.buf.frames_hint()
    }

    fn channels(&self) -> usize {
        self.buf.channels()
    }

    fn channel(&self, channel: usize) -> Channel<'_, T> {
        self.buf.channel(channel).tail(self.available)
    }
}
