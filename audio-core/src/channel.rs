//! A channel buffer as created through [Buf::get][crate::Buf::get] or
//! [BufMut::get_mut][crate::BufMut::get_mut].

/// One channel of audio samples, usually one of several channels in a multichannel buffer
///
/// This trait provides read-only access.
///
/// See [Buf::get][crate::Buf::get].
pub trait Channel {
    /// The sample of a channel.
    type Sample: Copy;

    /// The type the channel assumes when coerced into a reference.
    type Channel<'this>: Channel<Sample = Self::Sample>
    where
        Self: 'this;

    /// A borrowing iterator over the channel.
    type Iter<'this>: Iterator<Item = Self::Sample>
    where
        Self: 'this;

    /// Reborrow the current channel as a reference.
    fn as_channel(&self) -> Self::Channel<'_>;

    /// Get the length which indicates number of frames in the current channel.
    ///
    /// # Examples
    ///
    /// ```
    /// use audio::{Buf, Channel};
    ///
    /// fn test(buf: impl Buf<Sample = f32>) {
    ///     for chan in buf.iter() {
    ///         assert_eq!(chan.len(), 16);
    ///     }
    /// }
    ///
    /// test(&audio::dynamic![[0.0; 16]; 2]);
    /// test(&audio::sequential![[0.0; 16]; 2]);
    /// test(&audio::interleaved![[0.0; 16]; 2]);
    /// ```
    fn len(&self) -> usize;

    /// Test if the current channel is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use audio::{Buf, Channel};
    ///
    /// fn test(buf: impl Buf<Sample = f32>) {
    ///     for chan in buf.iter() {
    ///         assert!(!chan.is_empty());
    ///     }
    /// }
    ///
    /// test(&audio::dynamic![[0.0; 16]; 2]);
    /// test(&audio::sequential![[0.0; 16]; 2]);
    /// test(&audio::interleaved![[0.0; 16]; 2]);
    /// ```
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the frame at the given offset in the channel.
    ///
    /// # Examples
    ///
    /// ```
    /// use audio::{Buf, Channel};
    ///
    /// fn test(buf: impl Buf<Sample = f32>) {
    ///     for chan in buf.iter() {
    ///         assert_eq!(chan.get(15), Some(0.0));
    ///         assert_eq!(chan.get(16), None);
    ///     }
    /// }
    ///
    /// test(&audio::dynamic![[0.0; 16]; 2]);
    /// test(&audio::sequential![[0.0; 16]; 2]);
    /// test(&audio::interleaved![[0.0; 16]; 2]);
    /// ```
    fn get(&self, n: usize) -> Option<Self::Sample>;

    /// Construct an iterator over the channel.
    ///
    /// # Examples
    ///
    /// ```
    /// use audio::{Buf, BufMut, Channel, ChannelMut};
    ///
    /// let mut left = audio::interleaved![[0.0f32; 4]; 2];
    /// let mut right = audio::dynamic![[0.0f32; 4]; 2];
    ///
    /// if let (Some(mut left), Some(mut right)) = (left.get_mut(0), right.get_mut(0)) {
    ///     for (l, r) in left.iter_mut().zip(right.iter_mut()) {
    ///         *l = 1.0;
    ///         *r = 1.0;
    ///     }
    /// }
    ///
    /// assert!(left.get(0).unwrap().iter().eq(right.get(0).unwrap().iter()));
    ///
    /// assert_eq!(left.as_slice(), &[1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0]);
    /// assert_eq!(&right[0], &[1.0, 1.0, 1.0, 1.0]);
    /// assert_eq!(&right[1], &[0.0, 0.0, 0.0, 0.0]);
    /// ```
    fn iter(&self) -> Self::Iter<'_>;

    /// Try to access the current channel as a linear buffer.
    ///
    /// This is available because it could permit for some optimizations.
    ///
    /// # Examples
    ///
    /// ```
    /// use audio::{Buf, Channel};
    ///
    /// fn test(buf: &impl Buf<Sample = f32>, expected: Option<&[f32]>) {
    ///     assert_eq!(buf.get(0).unwrap().try_as_linear(), expected);
    /// }
    ///
    /// test(&audio::dynamic![[1.0; 16]; 2], Some(&[1.0; 16]));
    /// test(&audio::sequential![[1.0; 16]; 2], Some(&[1.0; 16]));
    /// test(&audio::interleaved![[1.0; 16]; 2], None);
    /// ```
    fn try_as_linear(&self) -> Option<&[Self::Sample]>;

    /// Construct a channel buffer where the first `n` frames are skipped.
    ///
    /// Skipping to the end of the buffer will result in an empty buffer.
    ///
    /// ```
    /// use audio::{Buf, Channel};
    ///
    /// let buf = audio::interleaved![[0; 4]; 2];
    ///
    /// for chan in buf.iter() {
    ///     assert_eq!(chan.skip(1).len(), 3);
    ///     assert_eq!(chan.skip(4).len(), 0);
    /// }
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use audio::{Buf, BufMut, Channel, ChannelMut};
    ///
    /// let mut from = audio::interleaved![[0.0f32; 4]; 2];
    /// *from.sample_mut(0, 2).unwrap() = 1.0;
    /// *from.sample_mut(0, 3).unwrap() = 1.0;
    ///
    /// let mut to = audio::interleaved![[0.0f32; 4]; 2];
    ///
    /// if let (Some(from), Some(to)) = (from.get(0), to.get_mut(0)) {
    ///     audio::channel::copy(from.skip(2), to);
    /// }
    ///
    /// assert_eq!(to.as_slice(), &[1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    /// ```
    fn skip(self, n: usize) -> Self;

    /// Construct a channel buffer where the last `n` frames are included.
    ///
    /// # Examples
    ///
    /// ```
    /// use audio::{Buf, BufMut, Channel, ChannelMut};
    ///
    /// let from = audio::interleaved![[1.0f32; 4]; 2];
    /// let mut to = audio::interleaved![[0.0f32; 4]; 2];
    ///
    /// if let (Some(from), Some(to)) = (from.get(0), to.get_mut(0)) {
    ///     audio::channel::copy(from, to.tail(2));
    /// }
    ///
    /// assert_eq!(to.as_slice(), &[0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0]);
    /// ```
    fn tail(self, n: usize) -> Self;

    /// Limit the channel bufferto `limit` number of frames.
    ///
    /// # Examples
    ///
    /// ```
    /// use audio::{Buf, BufMut, Channel, ChannelMut};
    ///
    /// let from = audio::interleaved![[1.0f32; 4]; 2];
    /// let mut to = audio::interleaved![[0.0f32; 4]; 2];
    ///
    /// if let (Some(from), Some(to)) = (from.get(0), to.get_mut(0)) {
    ///     audio::channel::copy(from.limit(2), to);
    /// }
    ///
    /// assert_eq!(to.as_slice(), &[1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    /// ```
    fn limit(self, limit: usize) -> Self;
}
