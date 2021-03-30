//! Utilities for manipulating audio buffers.

use crate::buf::{Buf, BufMut};
use crate::sample::{Sample, Translate};

/// Copy from the buffer specified by `from` into the buffer specified by `to`.
///
/// Only the common count of channels will be copied.
pub fn copy<I, O, T>(from: I, mut to: O)
where
    I: Buf<T>,
    O: BufMut<T>,
    T: Sample,
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
    I: Buf<U>,
    O: BufMut<T>,
    T: Sample,
    U: Sample,
    T: Translate<U>,
{
    let end = usize::min(from.channels(), to.channels());

    for chan in 0..end {
        to.channel_mut(chan).translate_from(from.channel(chan));
    }
}
