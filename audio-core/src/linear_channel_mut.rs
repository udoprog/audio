use crate::ChannelMut;

/// Trait for linear mutable channels.
pub trait LinearChannelMut: ChannelMut {
    /// Access the linear channel mutably.
    fn as_linear_channel_mut(&mut self) -> &mut [Self::Sample];
}
