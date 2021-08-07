//! Utilities for working with audio buffers.

use audio_core::{Buf, BufMut, Channel, ChannelMut, Translate};

/// Copy from the buffer specified by `from` into the buffer specified by `to`.
///
/// Only the common count of channels will be copied.
pub fn copy<I, O>(from: I, mut to: O)
where
    I: Buf,
    O: BufMut<Sample = I::Sample>,
    I::Sample: Copy,
{
    let end = usize::min(from.channels(), to.channels());

    for chan in 0..end {
        let mut to = to.channel_mut(chan);
        let from = from.channel(chan);
        to.copy_from(from);
    }
}

/// Translate the content of one buffer `from` into the buffer specified by `to`.
///
/// Only the common count of channels will be copied.
pub fn translate<I, O>(from: I, mut to: O)
where
    I: Buf,
    O: BufMut,
    O::Sample: Translate<I::Sample>,
    I::Sample: Copy,
{
    let end = usize::min(from.channels(), to.channels());

    for chan in 0..end {
        let mut to = to.channel_mut(chan);
        let from = from.channel(chan);

        for (t, f) in to.iter_mut().zip(from.iter()) {
            *t = O::Sample::translate(*f);
        }
    }
}
