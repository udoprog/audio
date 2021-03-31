//! Utilities for manipulating audio buffers.

use rotary_core::Translate;
use rotary_core::{Buf, BufMut, ReadBuf, WriteBuf};

/// Copy the shared remaining frames from `from` into `to`.
///
/// This will copy the minimum number of frames between [ReadBuf::remaining] and
/// [WriteBuf::remaining_mut], and advance the provided buffers appropriately
/// using [ReadBuf::advance] and [WriteBuf::advance_mut].
pub fn copy_remaining<I, O, T>(mut from: I, mut to: O)
where
    I: ReadBuf + Buf<T>,
    O: WriteBuf + BufMut<T>,
    T: Copy,
{
    let len = usize::min(from.remaining(), to.remaining_mut());
    crate::buf::copy(&from, &mut to);
    from.advance(len);
    to.advance_mut(len);
}

/// Translate the shared remaining frames from `from` into `to`.
///
/// Samples will be translated through the [Translate] trait.
///
/// This will translate the minimum number of frames between
/// [ReadBuf::remaining] and [WriteBuf::remaining_mut], and advance the provided
/// buffers appropriately using [ReadBuf::advance] and [WriteBuf::advance_mut].
pub fn translate_remaining<I, O, T, U>(mut from: I, mut to: O)
where
    U: Translate<T>,
    I: ReadBuf + Buf<T>,
    O: WriteBuf + BufMut<U>,
    T: Copy,
{
    let len = usize::min(from.remaining(), to.remaining_mut());
    crate::buf::translate(&from, &mut to);
    from.advance(len);
    to.advance_mut(len);
}
