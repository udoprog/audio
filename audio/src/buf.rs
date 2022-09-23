//! Utilities for working with audio buffers.

use core::{Buf, BufMut, Channel, ChannelMut, Translate};

pub mod dynamic;
pub use self::dynamic::Dynamic;

pub mod interleaved;
pub use self::interleaved::Interleaved;

pub mod sequential;
pub use self::sequential::Sequential;

/// Copy from the buffer specified by `from` into the buffer specified by `to`.
///
/// Only the common count of channels will be copied.
pub fn copy<I, O>(from: I, mut to: O)
where
    I: Buf,
    O: BufMut<Sample = I::Sample>,
    I::Sample: Copy,
{
    for (from, to) in from.iter().zip(to.iter_mut()) {
        crate::channel::copy(from, to);
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
    for (mut to, from) in to.iter_mut().zip(from.iter()) {
        for (t, f) in to.iter_mut().zip(from.iter()) {
            *t = O::Sample::translate(f);
        }
    }
}
