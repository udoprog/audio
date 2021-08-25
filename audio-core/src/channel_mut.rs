use crate::Channel;

/// The mutable buffer of a single channel.
///
/// This doesn't provide direct access to the underlying buffer, but rather
/// allows us to copy data usinga  number of utility functions.
///
/// See [BufMut::get_mut][crate::BufMut::get_mut].
pub trait ChannelMut: Channel {
    /// A mutable iterator over a channel.
    type IterMut<'a>: Iterator<Item = &'a mut Self::Sample>
    where
        Self::Sample: 'a;

    /// Construct a mutable iterator over the channel
    fn iter_mut(&mut self) -> Self::IterMut<'_>;

    /// Copy from the given slice.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::{BufMut, Channel, ChannelMut};
    ///
    /// fn test(buf: &mut impl BufMut<Sample = f32>) {
    ///     let is_linear = if let Some(linear) = buf.get_mut(0).unwrap().as_linear_mut() {
    ///         linear[0] = 1.0;
    ///         true
    ///     } else {
    ///         false
    ///     };
    ///
    ///     if is_linear {
    ///         assert_eq!(buf.get(0).and_then(|c| c.get(0)), Some(1.0));
    ///     }
    /// }
    ///
    /// test(&mut audio::dynamic![[0.0; 8]; 2]);
    /// test(&mut audio::sequential![[0.0; 8]; 2]);
    /// test(&mut audio::interleaved![[0.0; 8]; 2]);
    /// ```
    fn as_linear_mut(&mut self) -> Option<&mut [Self::Sample]>;

    /// Copy the content of one channel to another.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::{Buf, BufMut, ChannelMut};
    ///
    /// let from = audio::interleaved![[1.0f32; 4]; 2];
    /// let mut to = audio::buf::Interleaved::<f32>::with_topology(2, 4);
    ///
    /// to.get_mut(0).unwrap().copy_from(from.limit(2).get(0).unwrap());
    /// assert_eq!(to.as_slice(), &[1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    /// ```
    fn copy_from<I>(&mut self, from: I)
    where
        Self::Sample: Copy,
        I: Channel<Sample = Self::Sample>,
    {
        match (self.as_linear_mut(), from.as_linear()) {
            (Some(to), Some(from)) => {
                let len = usize::min(to.len(), from.len());
                to[..len].copy_from_slice(&from[..len]);
            }
            _ => {
                for (t, f) in self.iter_mut().zip(from.iter()) {
                    *t = f;
                }
            }
        }
    }

    /// Copy an iterator into a channel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::{Buf, BufMut, ChannelMut};
    ///
    /// let mut buffer = audio::buf::Interleaved::with_topology(2, 4);
    ///
    /// (&mut buffer).skip(2).get_mut(0).unwrap().copy_from_iter([1.0, 1.0]);
    ///
    /// assert_eq!(buffer.as_slice(), &[0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0])
    /// ```
    fn copy_from_iter<I>(&mut self, from: I)
    where
        I: IntoIterator<Item = Self::Sample>,
    {
        for (to, from) in self.iter_mut().zip(from) {
            *to = from;
        }
    }
}
