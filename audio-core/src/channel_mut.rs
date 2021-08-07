use crate::channel::Channel;

/// The mutable buffer of a single channel.
///
/// This doesn't provide direct access to the underlying buffer, but rather
/// allows us to copy data usinga  number of utility functions.
///
/// See [Channels::channel_mut][crate::Channels::channel_mut].
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
    /// use audio::{ChannelsMut, ChannelMut};
    ///
    /// fn test(buf: &mut dyn ChannelsMut<f32>) {
    ///     if let Some(linear) = buf.channel_mut(0).as_linear_mut() {
    ///         let mut out = vec![0.0; 8];
    ///         audio::buf::copy_channel_into_iter(buf.channel(0), &mut out);
    ///         assert_eq!(out, vec![1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0]);
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
    /// use audio::{Channels, ChannelsMut, ChannelMut};
    ///
    /// let from = audio::interleaved![[1.0f32; 4]; 2];
    /// let mut to = audio::Interleaved::<f32>::with_topology(2, 4);
    ///
    /// to.channel_mut(0).copy_from(from.interleaved_limit(2).channel(0));
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
                    *t = *f;
                }
            }
        }
    }

    /// Copy an iterator into a channel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::{Channels, ChannelsMut, ChannelMut};
    ///
    /// let mut buffer = audio::Interleaved::with_topology(2, 4);
    ///
    /// buffer.interleaved_skip_mut(2).channel_mut(0).copy_from_iter([1.0, 1.0]);
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
