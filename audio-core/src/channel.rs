//! A channel buffer as created through [Channels::channel][crate::Channels::channel] or
//! [ChannelsMut::channel_mut][crate::ChannelsMut::channel_mut].

/// The buffer of a single channel.
///
/// This doesn't provide direct access to the underlying buffer, but rather
/// allows us to copy data usinga  number of utility functions.
///
/// See [Channels::channel][crate::Channels::channel].
pub trait Channel<T> {
    /// A borrowing iterator over the channel.
    type Iter<'a>: Iterator<Item = &'a T>
    where
        T: 'a;

    /// Access the number of frames on the current channel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::Channels;
    ///
    /// fn test(buf: &dyn Channels<f32>) {
    ///     let left = buf.channel(0);
    ///     let right = buf.channel(1);
    ///
    ///     assert_eq!(left.frames(), 16);
    ///     assert_eq!(right.frames(), 16);
    /// }
    ///
    /// test(&audio::dynamic![[0.0; 16]; 2]);
    /// test(&audio::sequential![[0.0; 16]; 2]);
    /// test(&audio::interleaved![[0.0; 16]; 2]);
    /// ```
    fn frames(&self) -> usize;

    /// Construct an iterator over the channel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::{Channels as _, ChannelsMut as _};
    ///
    /// let mut left = audio::interleaved![[0.0f32; 4]; 2];
    /// let mut right = audio::dynamic![[0.0f32; 4]; 2];
    ///
    /// for (l, r) in left.channel_mut(0).iter_mut().zip(right.channel_mut(0)) {
    ///     *l = 1.0;
    ///     *r = 1.0;
    /// }
    ///
    /// assert!(left.channel(0).iter().eq(right.channel(0).iter()));
    ///
    /// assert_eq!(left.as_slice(), &[1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0]);
    /// assert_eq!(&right[0], &[1.0, 1.0, 1.0, 1.0]);
    /// assert_eq!(&right[1], &[0.0, 0.0, 0.0, 0.0]);
    /// ```
    fn iter(&self) -> Self::Iter<'_>;

    /// Construct a channel buffer where the first `n` frames are skipped.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::{Channels as _, ChannelsMut as _};
    ///
    /// let mut from = audio::interleaved![[0.0f32; 4]; 2];
    /// *from.frame_mut(0, 2).unwrap() = 1.0;
    /// *from.frame_mut(0, 3).unwrap() = 1.0;
    ///
    /// let mut to = audio::interleaved![[0.0f32; 4]; 2];
    ///
    /// to.channel_mut(0).copy_from(from.channel(0).skip(2));
    /// assert_eq!(to.as_slice(), &[1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    /// ```
    fn skip(self, n: usize) -> Self;

    /// Construct a channel buffer where the last `n` frames are included.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::{Channels as _, ChannelsMut as _};
    ///
    /// let from = audio::interleaved![[1.0f32; 4]; 2];
    /// let mut to = audio::interleaved![[0.0f32; 4]; 2];
    ///
    /// to.channel_mut(0).as_mut().tail(2).copy_from(from.channel(0));
    /// assert_eq!(to.as_slice(), &[0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0]);
    /// ```
    fn tail(self, n: usize) -> Self;

    /// Limit the channel bufferto `limit` number of frames.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::{Channels as _, ChannelsMut as _};
    ///
    /// let from = audio::interleaved![[1.0f32; 4]; 2];
    /// let mut to = audio::interleaved![[0.0f32; 4]; 2];
    ///
    /// to.channel_mut(0).copy_from(from.channel(0).limit(2));
    /// assert_eq!(to.as_slice(), &[1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    /// ```
    fn limit(self, limit: usize) -> Self;

    /// Construct a range of frames corresponds to the chunk with `len` and
    /// position `n`.
    ///
    /// Which is the range `n * len .. n * len + len`.
    fn chunk(self, n: usize, len: usize) -> Self;

    /// How many chunks of the given size can you divide buf into.
    ///
    /// This includes one extra chunk even if the chunk doesn't divide the frame
    /// length evenly.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::Channels;
    ///
    /// fn test(buf: &dyn Channels<f32>) {
    ///     let left = buf.channel(0);
    ///     let right = buf.channel(1);
    ///
    ///     assert_eq!(left.chunks(4), 4);
    ///     assert_eq!(right.chunks(4), 4);
    ///
    ///     assert_eq!(left.chunks(6), 3);
    ///     assert_eq!(right.chunks(6), 3);
    /// }
    ///
    /// test(&audio::dynamic![[0.0; 16]; 2]);
    /// test(&audio::sequential![[0.0; 16]; 2]);
    /// test(&audio::interleaved![[0.0; 16]; 2]);
    /// ```
    fn chunks(&self, chunk: usize) -> usize;

    /// Try to coerce the channel the channel into a linear sequence of memory.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::{Channel, Channels};
    ///
    /// fn test(buf: &dyn Channels<f32>, expected: Option<&[f32]>) {
    ///     assert_eq!(buf.channel(0).as_linear(), expected);
    /// }
    ///
    /// test(&audio::dynamic![[1.0; 16]; 2], Some(&[1.0; 16]));
    /// test(&audio::sequential![[1.0; 16]; 2], Some(&[1.0; 16]));
    /// test(&audio::interleaved![[1.0; 16]; 2], None);
    /// ```
    fn as_linear(&self) -> Option<&[T]>;
}
