use crate::Channel;

/// One channel of audio samples, usually one of several channels in a multichannel buffer
///
/// This trait provides read and write access.
///
/// See [BufMut::get_channel_mut][crate::BufMut::get_channel_mut].
pub trait ChannelMut: Channel {
    /// A reborrowed mutable channel.
    type ChannelMut<'this>: ChannelMut<Sample = Self::Sample>
    where
        Self: 'this;

    /// A mutable iterator over a channel.
    type IterMut<'this>: Iterator<Item = &'this mut Self::Sample>
    where
        Self: 'this;

    /// Reborrow the channel mutably.
    fn as_channel_mut(&mut self) -> Self::ChannelMut<'_>;

    /// Construct a mutable iterator over the channel
    fn iter_mut(&mut self) -> Self::IterMut<'_>;

    /// Get the frame at the given offset in the channel.
    ///
    /// # Examples
    ///
    /// ```
    /// use audio::{BufMut, ChannelMut};
    ///
    /// fn test(mut buf: impl BufMut<Sample = i16>) {
    ///     for mut chan in buf.iter_channels_mut() {
    ///         if let Some(f) = chan.get_mut(2) {
    ///             *f = 1;
    ///         }
    ///     }
    /// }
    ///
    /// test(&mut audio::dynamic![[0; 16]; 2]);
    /// test(&mut audio::sequential![[0; 16]; 2]);
    /// test(&mut audio::interleaved![[0; 16]; 2]);
    /// ```
    fn get_mut(&mut self, n: usize) -> Option<&mut Self::Sample>;

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
    ///     let is_linear = if let Some(linear) = buf.get_channel_mut(0).unwrap().try_as_linear_mut() {
    ///         linear[2] = 1.0;
    ///         true
    ///     } else {
    ///         false
    ///     };
    ///
    ///     if is_linear {
    ///         assert_eq!(buf.get_channel(0).and_then(|c| c.get(2)), Some(1.0));
    ///     }
    /// }
    ///
    /// test(&mut audio::dynamic![[0.0; 8]; 2]);
    /// test(&mut audio::sequential![[0.0; 8]; 2]);
    /// test(&mut audio::interleaved![[0.0; 8]; 2]);
    /// ```
    fn try_as_linear_mut(&mut self) -> Option<&mut [Self::Sample]>;

    /// Replace all samples in the channel with the specified value
    ///
    /// # Example
    ///
    /// ```
    /// use audio::ChannelMut;
    ///
    /// let mut buf = audio::sequential![[0; 2]; 2];
    /// for mut channel in buf.iter_channels_mut() {
    ///     channel.fill(1);
    /// }
    /// assert_eq!(buf.get_channel(0).unwrap().as_ref(), &[1, 1]);
    /// assert_eq!(buf.get_channel(1).unwrap().as_ref(), &[1, 1]);
    /// ```
    fn fill(&mut self, value: Self::Sample) {
        for sample in self.iter_mut() {
            *sample = value;
        }
    }
}
