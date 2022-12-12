//! A dynamically sized, multi-channel interleaved audio buffer.

mod iter;
pub use self::iter::{IterChannels, IterChannelsMut};

#[cfg(feature = "std")]
mod buf;
#[cfg(feature = "std")]
pub use self::buf::Interleaved;
