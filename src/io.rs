//! Reading and writing sequentially from buffers.
//!
//! This is called buffered I/O, and allow buffers to support sequential reading
//! and writing to and from buffer.

use crate::buf::Buf;
use crate::sample::Sample;
use crate::translate::Translate;

#[cfg(test)]
mod tests;

mod read;
pub use self::read::Read;

mod write;
pub use self::write::Write;

mod read_write;
pub use self::read_write::ReadWrite;

/// A buffer that can keep track of how much has been read from it.
pub trait ReadBuf {
    /// Test if this buffer has remaining frames.
    fn has_remaining(&self) -> bool {
        self.remaining() > 0
    }

    /// The number of frames remaining in the readable buffer.
    fn remaining(&self) -> usize;

    /// Advance the read number of frames by `n`.
    fn advance(&mut self, n: usize);
}

impl<B> ReadBuf for &'_ mut B
where
    B: ReadBuf,
{
    fn has_remaining(&self) -> bool {
        (**self).has_remaining()
    }

    fn remaining(&self) -> usize {
        (**self).remaining()
    }

    fn advance(&mut self, n: usize) {
        (**self).advance(n);
    }
}

/// A buffer that can be written to.
pub trait WriteBuf<T>
where
    T: Sample,
{
    /// Test if this buffer has remaining mutable frames.
    fn has_remaining_mut(&self) -> bool {
        self.remaining_mut() > 0
    }

    /// Remaining number of frames that can be written.
    fn remaining_mut(&self) -> usize;

    /// Read frames from the given read buffer into this buffer.
    ///
    /// Advances the read from buffer by the number of frames read through
    /// [Read::advance].
    fn copy<I>(&mut self, buf: I)
    where
        I: ReadBuf + Buf<T>;

    /// Read translated frames from the given read buffer into this buffer.
    ///
    /// Advances the read from buffer by the number of frames read through
    /// [Read::advance].
    fn translate<I, U>(&mut self, buf: I)
    where
        T: Translate<U>,
        I: ReadBuf + Buf<U>,
        U: Sample;
}
