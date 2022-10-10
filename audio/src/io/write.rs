use audio_core::{Buf, BufMut, Channel, ExactSizeBuf, WriteBuf};

/// Make a mutable buffer into a write adapter that implements [WriteBuf].
///
/// # Examples
///
/// ```
/// use audio::{Buf, ReadBuf, WriteBuf};
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
    /// ```
    /// use audio::{WriteBuf, ExactSizeBuf};
    /// use audio::io;
    ///
    /// let buf = audio::interleaved![[1, 2, 3, 4], [5, 6, 7, 8]];
    /// assert_eq!(buf.frames(), 4);
    ///
    /// let buf = io::Write::new(buf);
    ///
    /// assert!(buf.has_remaining_mut());
    /// assert_eq!(buf.remaining_mut(), 4);
    /// ```
    #[inline]
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
    /// ```
    /// use audio::{WriteBuf, ExactSizeBuf};
    /// use audio::io;
    ///
    /// let buf = audio::interleaved![[1, 2, 3, 4], [5, 6, 7, 8]];
    /// assert_eq!(buf.frames(), 4);
    ///
    /// let buf = io::Write::empty(buf);
    ///
    /// assert!(!buf.has_remaining_mut());
    /// assert_eq!(buf.remaining_mut(), 0);
    /// ```
    #[inline]
    pub fn empty(buf: B) -> Self {
        Self { buf, available: 0 }
    }

    /// Access the underlying buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use audio::{io, wrap};
    ///
    /// let buf: audio::buf::Interleaved<i16> = audio::interleaved![[1, 2, 3, 4]; 4];
    /// let mut buf = io::Write::new(buf);
    ///
    /// io::copy_remaining(wrap::interleaved(&[0i16; 16][..], 4), &mut buf);
    ///
    /// assert_eq!(buf.as_ref().channels(), 4);
    /// ```
    #[inline]
    pub fn as_ref(&self) -> &B {
        &self.buf
    }

    /// Access the underlying buffer mutably.
    ///
    /// # Examples
    ///
    /// ```
    /// use audio::{io, wrap};
    /// use audio::Buf;
    ///
    /// let buf: audio::buf::Interleaved<i16> = audio::interleaved![[1, 2, 3, 4]; 4];
    /// let mut buf = io::Write::new(buf);
    ///
    /// io::copy_remaining(wrap::interleaved(&[0i16; 16][..], 4), &mut buf);
    ///
    /// buf.as_mut().resize_channels(2);
    ///
    /// assert_eq!(buf.channels(), 2);
    /// ```
    #[inline]
    pub fn as_mut(&mut self) -> &mut B {
        &mut self.buf
    }

    /// Convert into the underlying buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use audio::Buf;
    /// use audio::io;
    ///
    /// let buf: audio::buf::Interleaved<i16> = audio::interleaved![[1, 2, 3, 4]; 4];
    /// let mut buf = io::Write::new(buf);
    ///
    /// io::copy_remaining(audio::wrap::interleaved(&[0i16; 16][..], 4), &mut buf);
    ///
    /// let buf = buf.into_inner();
    ///
    /// assert_eq!(buf.channels(), 4);
    /// ```
    #[inline]
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
    /// ```
    /// use audio::{BufMut, WriteBuf};
    /// use audio::io;
    ///
    /// fn write_to_buf(mut write: impl BufMut<Sample = i16> + WriteBuf) {
    ///     let mut from = audio::interleaved![[0; 4]; 2];
    ///     io::copy_remaining(io::Read::new(&mut from), write);
    /// }
    ///
    /// let mut buf = io::Write::new(audio::interleaved![[1, 2, 3, 4], [5, 6, 7, 8]]);
    /// write_to_buf(&mut buf);
    ///
    /// assert!(!buf.has_remaining_mut());
    ///
    /// buf.set_written(0);
    ///
    /// assert!(buf.has_remaining_mut());
    /// ```
    #[inline]
    pub fn set_written(&mut self, written: usize)
    where
        B: ExactSizeBuf,
    {
        self.available = self.buf.frames().saturating_sub(written);
    }
}

impl<B> Write<B>
where
    B: Buf,
{
    /// Construct an iterator over all available channels.
    #[inline]
    pub fn iter_channels(&self) -> IterChannels<B> {
        IterChannels {
            iter: self.buf.iter_channels(),
            available: self.available,
        }
    }
}

impl<B> Write<B>
where
    B: BufMut,
{
    /// Construct a mutable iterator over all available channels.
    #[inline]
    pub fn iter_channels_mut(&mut self) -> IterChannelsMut<B> {
        IterChannelsMut {
            iter: self.buf.iter_channels_mut(),
            available: self.available,
        }
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
    #[inline]
    fn frames(&self) -> usize {
        self.buf.frames()
    }
}

impl<B> Buf for Write<B>
where
    B: Buf,
{
    type Sample = B::Sample;

    type Channel<'this> = B::Channel<'this>
    where
        Self: 'this;

    type IterChannels<'this> = IterChannels<'this, B>
    where
        Self: 'this;

    type Frame<'this> = B::Frame<'this>
    where
        Self: 'this;

    type IterFrames<'this> = B::IterFrames<'this>
    where
        Self: 'this;

    #[inline]
    fn frames_hint(&self) -> Option<usize> {
        self.buf.frames_hint()
    }

    #[inline]
    fn channels(&self) -> usize {
        self.buf.channels()
    }

    #[inline]
    fn channel(&self, channel: usize) -> Option<Self::Channel<'_>> {
        Some(self.buf.channel(channel)?.tail(self.available))
    }

    #[inline]
    fn iter_channels(&self) -> Self::IterChannels<'_> {
        (*self).iter_channels()
    }

    #[inline]
    fn frame(&self, frame: usize) -> Option<Self::Frame<'_>> {
        todo!()
    }

    #[inline]
    fn iter_frames(&self) -> Self::IterFrames<'_> {
        todo!()
    }
}

impl<B> BufMut for Write<B>
where
    B: BufMut,
{
    type ChannelMut<'this> = B::ChannelMut<'this>
    where
        Self: 'this;

    type IterChannelsMut<'this> = IterChannelsMut<'this, B>
    where
        Self: 'this;

    #[inline]
    fn channel_mut(&mut self, channel: usize) -> Option<Self::ChannelMut<'_>> {
        Some(self.buf.channel_mut(channel)?.tail(self.available))
    }

    #[inline]
    fn copy_channel(&mut self, from: usize, to: usize)
    where
        Self::Sample: Copy,
    {
        self.buf.copy_channel(from, to);
    }

    #[inline]
    fn iter_channels_mut(&mut self) -> Self::IterChannelsMut<'_> {
        (*self).iter_channels_mut()
    }
}

iter! {
    IterChannels,
    available: usize,
    =>
    self.tail(available)
}

iter_mut! {
    IterChannelsMut,
    available: usize,
    =>
    self.tail(available)
}
