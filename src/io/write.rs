use crate::buf::{Buf, BufInfo, BufMut};
use crate::channel::{Channel, ChannelMut};
use crate::io::{ReadBuf, WriteBuf};
use crate::sample::Sample;
use crate::translate::Translate;

/// Make a mutable buffer into a write adapter that implements
/// [WriteBuf].
///
/// # Examples
///
/// ```rust
/// use rotary::{Buf as _, BufMut as _, ReadBuf as _, WriteBuf as _};
/// use rotary::io::{Read, Write};
///
/// let from = rotary::interleaved![[1.0f32, 2.0f32, 3.0f32, 4.0f32]; 2];
/// let to = rotary::interleaved![[0.0f32; 4]; 2];
/// let mut to = Write::new(to);
/// let mut from = Read::new(from.skip(2));
///
/// assert_eq!(to.remaining_mut(), 4);
/// to.copy(from);
/// assert_eq!(to.remaining_mut(), 2);
///
/// assert_eq! {
///     to.as_ref().as_slice(),
///     &[3.0, 3.0, 4.0, 4.0, 0.0, 0.0, 0.0, 0.0],
/// };
/// ```
pub struct Write<B> {
    buf: B,
    available: usize,
}

impl<B> Write<B>
where
    B: BufInfo,
{
    /// Construct a new write adapter.
    pub fn new(buf: B) -> Self {
        let available = buf.buf_info_frames();
        Self { buf, available }
    }

    /// Access the underlying buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::io::Write;
    /// use rotary::{Buf as _, WriteBuf as _};
    ///
    /// let buffer: rotary::Interleaved<i16> = rotary::interleaved![[1, 2, 3, 4]; 4];
    /// let mut buffer = Write::new(buffer);
    ///
    /// buffer.copy(rotary::wrap::interleaved(&[0i16; 16][..], 4));
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
    /// use rotary::io::Write;
    /// use rotary::{Buf as _, WriteBuf as _};
    ///
    /// let buffer: rotary::Interleaved<i16> = rotary::interleaved![[1, 2, 3, 4]; 4];
    /// let mut buffer = Write::new(buffer);
    ///
    /// buffer.copy(rotary::wrap::interleaved(&[0i16; 16][..], 4));
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
    /// use rotary::io::Write;
    /// use rotary::{Buf as _, WriteBuf as _};
    ///
    /// let buffer: rotary::Interleaved<i16> = rotary::interleaved![[1, 2, 3, 4]; 4];
    /// let mut buffer = Write::new(buffer);
    ///
    /// buffer.copy(rotary::wrap::interleaved(&[0i16; 16][..], 4));
    ///
    /// let buffer = buffer.into_inner();
    ///
    /// assert_eq!(buffer.channels(), 4);
    /// ```
    pub fn into_inner(self) -> B {
        self.buf
    }
}

impl<B, T> WriteBuf<T> for Write<B>
where
    B: BufMut<T>,
    T: Sample,
{
    /// Remaining number of frames available.
    fn remaining_mut(&self) -> usize {
        self.available
    }

    /// Write to the underlying buffer.
    fn copy<I>(&mut self, mut buf: I)
    where
        I: ReadBuf + Buf<T>,
    {
        let len = usize::min(self.available, buf.buf_info_frames());
        crate::utils::copy(&buf, (&mut self.buf).tail(self.available));
        self.available = self.available.saturating_sub(len);
        buf.advance(len);
    }

    /// Write translated samples to the underlying buffer.
    fn translate<I, U>(&mut self, mut buf: I)
    where
        T: Translate<U>,
        I: ReadBuf + Buf<U>,
        U: Sample,
    {
        let len = usize::min(self.available, buf.remaining());
        crate::utils::translate(&buf, (&mut self.buf).tail(self.available));
        self.available = self.available.saturating_sub(len);
        buf.advance(len);
    }
}

impl<B> BufInfo for Write<B>
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

impl<B, T> Buf<T> for Write<B>
where
    B: Buf<T>,
    T: Sample,
{
    fn channel(&self, channel: usize) -> Channel<'_, T> {
        self.buf.channel(channel)
    }
}

impl<B, T> BufMut<T> for Write<B>
where
    B: BufMut<T>,
    T: Sample,
{
    fn channel_mut(&mut self, channel: usize) -> ChannelMut<'_, T> {
        self.buf.channel_mut(channel).tail(self.available)
    }
}
