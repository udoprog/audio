//! A channel buffer as created through [Buf::channel][crate::Buf::channel] or
//! [BufMut::channel_mut][crate::BufMut::channel_mut].

/// The buffer of a single channel.
///
/// This doesn't provide direct access to the underlying buffer, but rather
/// allows us to copy data usinga  number of utility functions.
///
/// See [Buf::channel][crate::Buf::channel].
pub trait Channel {
    /// The sample of a channel.
    type Sample: Copy;

    /// A borrowing iterator over the channel.
    type Iter<'a>: Iterator<Item = Self::Sample>
    where
        Self::Sample: 'a;

    /// Access the number of frames on the current channel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::{Buf, Channel};
    ///
    /// fn test(buf: &impl Buf<Sample = f32>) {
    ///     let left = buf.get(0).unwrap();
    ///     let right = buf.get(1).unwrap();
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

    /// Get the frame at the given offset.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::{Buf, Channel};
    ///
    /// fn test(buf: &impl Buf<Sample = f32>) {
    ///     let left = buf.get(0).unwrap();
    ///     let right = buf.get(1).unwrap();
    ///
    ///     assert_eq!(left.get(0), Some(0.0));
    ///     assert_eq!(right.get(0), Some(0.0));
    /// }
    ///
    /// test(&audio::dynamic![[0.0; 16]; 2]);
    /// test(&audio::sequential![[0.0; 16]; 2]);
    /// test(&audio::interleaved![[0.0; 16]; 2]);
    /// ```
    fn get(&self, n: usize) -> Option<Self::Sample> {
        self.iter().nth(n)
    }

    /// Construct an iterator over the channel.
    ///
    /// # Examples
    ///
    /// ```rust
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

    /// Construct a channel buffer where the first `n` frames are skipped.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::{Buf, BufMut, Channel, ChannelMut};
    ///
    /// let mut from = audio::interleaved![[0.0f32; 4]; 2];
    /// *from.frame_mut(0, 2).unwrap() = 1.0;
    /// *from.frame_mut(0, 3).unwrap() = 1.0;
    ///
    /// let mut to = audio::interleaved![[0.0f32; 4]; 2];
    ///
    /// if let (Some(mut to), Some(from)) = (to.get_mut(0), from.get(0)) {
    ///     to.copy_from(from.skip(2));
    /// }
    ///
    /// assert_eq!(to.as_slice(), &[1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    /// ```
    fn skip(self, n: usize) -> Self;

    /// Construct a channel buffer where the last `n` frames are included.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::{Buf, BufMut, Channel, ChannelMut};
    ///
    /// let from = audio::interleaved![[1.0f32; 4]; 2];
    /// let mut to = audio::interleaved![[0.0f32; 4]; 2];
    ///
    /// if let (Some(mut to), Some(from)) = (to.get_mut(0), from.get(0)) {
    ///     to.tail(2).copy_from(from);
    /// }
    ///
    /// assert_eq!(to.as_slice(), &[0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0]);
    /// ```
    fn tail(self, n: usize) -> Self;

    /// Limit the channel bufferto `limit` number of frames.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::{Buf, BufMut, Channel, ChannelMut};
    ///
    /// let from = audio::interleaved![[1.0f32; 4]; 2];
    /// let mut to = audio::interleaved![[0.0f32; 4]; 2];
    ///
    /// if let (Some(mut to), Some(from)) = (to.get_mut(0), from.get(0)) {
    ///     to.copy_from(from.limit(2));
    /// }
    ///
    /// assert_eq!(to.as_slice(), &[1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    /// ```
    fn limit(self, limit: usize) -> Self;

    /// Construct a range of frames corresponding to the chunk with the window
    /// size `window` at position `n`.
    ///
    /// Which is the range `n * window .. n * window + window`.
    fn chunk(self, n: usize, window: usize) -> Self;

    /// How many chunks of the given size can you divide buf into.
    ///
    /// This includes one extra chunk even if the chunk doesn't divide the frame
    /// length evenly.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::{Buf, Channel};
    ///
    /// fn test(buf: &impl Buf<Sample = f32>) {
    ///     let left = buf.get(0).unwrap();
    ///     let right = buf.get(1).unwrap();
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
    fn chunks(&self, chunk: usize) -> usize {
        let len = self.frames();

        if len % chunk == 0 {
            len / chunk
        } else {
            len / chunk + 1
        }
    }

    /// Try to coerce the channel the channel into a linear sequence of memory.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::{Buf, Channel};
    ///
    /// fn test(buf: &impl Buf<Sample = f32>, expected: Option<&[f32]>) {
    ///     assert_eq!(buf.get(0).unwrap().as_linear(), expected);
    /// }
    ///
    /// test(&audio::dynamic![[1.0; 16]; 2], Some(&[1.0; 16]));
    /// test(&audio::sequential![[1.0; 16]; 2], Some(&[1.0; 16]));
    /// test(&audio::interleaved![[1.0; 16]; 2], None);
    /// ```
    fn as_linear(&self) -> Option<&[Self::Sample]>;

    /// Copy the contents of a channel into an iterator.
    fn copy_into_iter<'a, I>(&self, to: I)
    where
        Self::Sample: 'a + Copy,
        I: IntoIterator<Item = &'a mut Self::Sample>,
    {
        for (from, to) in self.iter().zip(to) {
            *to = from;
        }
    }
}
