use crate::channel::Channel;

/// The mutable buffer of a single channel.
///
/// This doesn't provide direct access to the underlying buffer, but rather
/// allows us to copy data usinga  number of utility functions.
///
/// See [Channels::channel_mut][crate::Channels::channel_mut].
pub trait ChannelMut<T>: Channel<T> {
    /// A mutable iterator over a channel.
    type IterMut<'a>: Iterator<Item = &'a mut T>
    where
        T: 'a;

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
    fn as_linear_mut(&mut self) -> Option<&mut [T]>;
}
