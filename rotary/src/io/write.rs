use rotary_core::{Buf, Channel, ChannelMut, Channels, ChannelsMut, ExactSizeBuf, WriteBuf};

/// Make a mutable buffer into a write adapter that implements [WriteBuf].
///
/// # Examples
///
/// ```rust
/// use rotary::{Buf as _, ReadBuf as _, WriteBuf as _};
/// use rotary::io;
///
/// let from = rotary::interleaved![[1.0f32, 2.0f32, 3.0f32, 4.0f32]; 2];
/// let mut from = io::Read::new(from.skip(2));
///
/// let to = rotary::interleaved![[0.0f32; 4]; 2];
/// let mut to = io::Write::new(to);
///
/// assert_eq!(to.remaining_mut(), 4);
/// io::copy_remaining(from, &mut to);
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
    B: ExactSizeBuf,
{
    /// Construct a new write adapter.
    pub fn new(buf: B) -> Self {
        let available = buf.frames();
        Self { buf, available }
    }

    /// Access the underlying buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::{io, wrap};
    ///
    /// let buffer: rotary::Interleaved<i16> = rotary::interleaved![[1, 2, 3, 4]; 4];
    /// let mut buffer = io::Write::new(buffer);
    ///
    /// io::copy_remaining(wrap::interleaved(&[0i16; 16][..], 4), &mut buffer);
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
    /// use rotary::{io, wrap};
    /// use rotary::Buf as _;
    ///
    /// let buffer: rotary::Interleaved<i16> = rotary::interleaved![[1, 2, 3, 4]; 4];
    /// let mut buffer = io::Write::new(buffer);
    ///
    /// io::copy_remaining(wrap::interleaved(&[0i16; 16][..], 4), &mut buffer);
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
    /// use rotary::Buf as _;
    /// use rotary::io;
    ///
    /// let buffer: rotary::Interleaved<i16> = rotary::interleaved![[1, 2, 3, 4]; 4];
    /// let mut buffer = io::Write::new(buffer);
    ///
    /// io::copy_remaining(rotary::wrap::interleaved(&[0i16; 16][..], 4), &mut buffer);
    ///
    /// let buffer = buffer.into_inner();
    ///
    /// assert_eq!(buffer.channels(), 4);
    /// ```
    pub fn into_inner(self) -> B {
        self.buf
    }
}

impl<B> WriteBuf for Write<B> {
    /// Remaining number of frames available.
    #[inline]
    fn remaining_mut(&self) -> usize {
        self.available
    }

    #[inline]
    fn advance_mut(&mut self, n: usize) {
        self.available = self.available.saturating_sub(n);
    }
}

impl<B> ExactSizeBuf for Write<B>
where
    B: ExactSizeBuf,
{
    fn frames(&self) -> usize {
        self.buf.frames()
    }
}

impl<B> Buf for Write<B>
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

impl<B, T> Channels<T> for Write<B>
where
    B: Channels<T>,
{
    fn channel(&self, channel: usize) -> Channel<'_, T> {
        self.buf.channel(channel).tail(self.available)
    }
}

impl<B, T> ChannelsMut<T> for Write<B>
where
    B: ChannelsMut<T>,
{
    #[inline]
    fn channel_mut(&mut self, channel: usize) -> ChannelMut<'_, T> {
        self.buf.channel_mut(channel).tail(self.available)
    }

    #[inline]
    fn copy_channels(&mut self, from: usize, to: usize)
    where
        T: Copy,
    {
        self.buf.copy_channels(from, to);
    }
}
