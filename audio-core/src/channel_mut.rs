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

    /// Try to access the current channel as a mutable linear buffer.
    ///
    /// This is available because it could permit for some optimizations.
    ///
    /// # Examples
    ///
    /// ```
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
}
