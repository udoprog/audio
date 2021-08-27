use crate::Channel;

/// Traits for linear channels.
pub trait LinearChannel: Channel {
    /// Access the linear channel.
    fn as_linear_channel(&self) -> &[Self::Sample];
}
