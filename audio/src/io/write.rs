use audio_core::{Buf, Channel, Channels, ChannelsMut, ExactSizeBuf, WriteBuf};

/// Make a mutable buffer into a write adapter that implements [WriteBuf].
///
/// # Examples
///
/// ```rust
/// use audio::{Buf as _, ReadBuf as _, WriteBuf as _};
/// use audio::io;
///
/// let from = audio::interleaved![[1.0f32, 2.0f32, 3.0f32, 4.0f32]; 2];
/// let mut from = io::Read::new(from.skip(2));
///
/// let to = audio::interleaved![[0.0f32; 4]; 2];
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

impl<B> Write<B> {
    /// Construct a new writing adapter.
    ///
    /// The constructed writer will be initialized so that the number of bytes
    /// available for writing are equal to what's reported by
    /// [ExactSizeBuf::frames].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::{WriteBuf, ExactSizeBuf};
    /// use audio::io;
    ///
    /// let buffer = audio::interleaved![[1, 2, 3, 4], [5, 6, 7, 8]];
    /// assert_eq!(buffer.frames(), 4);
    ///
    /// let buffer = io::Write::new(buffer);
    ///
    /// assert!(buffer.has_remaining_mut());
    /// assert_eq!(buffer.remaining_mut(), 4);
    /// ```
    pub fn new(buf: B) -> Self
    where
        B: ExactSizeBuf,
    {
        let available = buf.frames();
        Self { buf, available }
    }

    /// Construct a new writing adapter.
    ///
    /// The constructed reader will be initialized so that there are no frames
    /// available to be written.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::{WriteBuf, ExactSizeBuf};
    /// use audio::io;
    ///
    /// let buffer = audio::interleaved![[1, 2, 3, 4], [5, 6, 7, 8]];
    /// assert_eq!(buffer.frames(), 4);
    ///
    /// let buffer = io::Write::empty(buffer);
    ///
    /// assert!(!buffer.has_remaining_mut());
    /// assert_eq!(buffer.remaining_mut(), 0);
    /// ```
    pub fn empty(buf: B) -> Self {
        Self { buf, available: 0 }
    }

    /// Access the underlying buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::{io, wrap};
    ///
    /// let buffer: audio::Interleaved<i16> = audio::interleaved![[1, 2, 3, 4]; 4];
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
    /// use audio::{io, wrap};
    /// use audio::Buf as _;
    ///
    /// let buffer: audio::Interleaved<i16> = audio::interleaved![[1, 2, 3, 4]; 4];
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
    /// use audio::Buf as _;
    /// use audio::io;
    ///
    /// let buffer: audio::Interleaved<i16> = audio::interleaved![[1, 2, 3, 4]; 4];
    /// let mut buffer = io::Write::new(buffer);
    ///
    /// io::copy_remaining(audio::wrap::interleaved(&[0i16; 16][..], 4), &mut buffer);
    ///
    /// let buffer = buffer.into_inner();
    ///
    /// assert_eq!(buffer.channels(), 4);
    /// ```
    pub fn into_inner(self) -> B {
        self.buf
    }

    /// Set the number of frames written.
    ///
    /// This can be used to rewind the internal cursor to a previously written
    /// frame if needed. Or, if the underlying buffer has changed for some
    /// reason, like if it was read into through a call to [Write::as_mut].
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
    /// let mut buffer = io::Write::new(audio::interleaved![[1, 2, 3, 4], [5, 6, 7, 8]]);
    /// write_to_buf(&mut buffer);
    ///
    /// assert!(!buffer.has_remaining_mut());
    ///
    /// buffer.set_written(0);
    ///
    /// assert!(buffer.has_remaining_mut());
    /// ```
    #[inline]
    pub fn set_written(&mut self, written: usize)
    where
        B: ExactSizeBuf,
    {
        self.available = self.buf.frames().saturating_sub(written);
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

impl<B> Channels for Write<B>
where
    B: Channels,
{
    type Sample = B::Sample;

    type Channel<'a>
    where
        Self::Sample: 'a,
    = B::Channel<'a>;

    fn channel(&self, channel: usize) -> Self::Channel<'_> {
        self.buf.channel(channel).tail(self.available)
    }
}

impl<B> ChannelsMut for Write<B>
where
    B: ChannelsMut,
{
    type ChannelMut<'a>
    where
        Self::Sample: 'a,
    = B::ChannelMut<'a>;

    #[inline]
    fn channel_mut(&mut self, channel: usize) -> Self::ChannelMut<'_> {
        self.buf.channel_mut(channel).tail(self.available)
    }

    #[inline]
    fn copy_channels(&mut self, from: usize, to: usize)
    where
        Self::Sample: Copy,
    {
        self.buf.copy_channels(from, to);
    }
}
