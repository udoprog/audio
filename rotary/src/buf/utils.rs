//! Utilities for manipulating audio buffers.

use rotary_core::Translate;
use rotary_core::{Channels, ChannelsMut};

/// Copy from the buffer specified by `from` into the buffer specified by `to`.
///
/// Only the common count of channels will be copied.
pub fn copy<I, O, T>(from: I, mut to: O)
where
    I: Channels<T>,
    O: ChannelsMut<T>,
    T: Copy,
{
    let end = usize::min(from.channels(), to.channels());

    for chan in 0..end {
        to.channel_mut(chan).copy_from(from.channel(chan));
    }
}

/// Translate the content of one buffer `from` into the buffer specified by `to`.
///
/// Only the common count of channels will be copied.
pub fn translate<I, O, U, T>(from: I, mut to: O)
where
    I: Channels<U>,
    O: ChannelsMut<T>,
    T: Translate<U>,
    U: Copy,
{
    let end = usize::min(from.channels(), to.channels());

    for chan in 0..end {
        to.channel_mut(chan).translate_from(from.channel(chan));
    }
}
