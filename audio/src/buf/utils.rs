//! Utilities for manipulating audio buffers.

use audio_core::{Channel, ChannelMut, Channels, ChannelsMut, Translate};

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
        let mut to = to.channel_mut(chan);
        let from = from.channel(chan);

        match (to.as_linear_mut(), from.as_linear()) {
            (Some(to), Some(from)) => {
                let len = usize::min(to.len(), from.len());
                to[..len].copy_from_slice(&from[..len]);
            }
            _ => {
                for (t, f) in to.iter_mut().zip(from.iter()) {
                    *t = *f;
                }
            }
        }
    }
}

/// Copy a channel into an iterator.
pub fn copy_channel_into_iter<'a, I, O, T>(from: I, out: O)
where
    I: Channel<T>,
    O: IntoIterator<Item = &'a mut T>,
    T: 'a + Copy,
{
    for (from, to) in from.iter().zip(out) {
        *to = *from;
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
        let mut to = to.channel_mut(chan);
        let from = from.channel(chan);

        for (t, f) in to.iter_mut().zip(from.iter()) {
            *t = T::translate(*f);
        }
    }
}
