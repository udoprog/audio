use crate::buf::{Buf, BufInfo, BufMut};
use crate::io::{ReadBuf, WriteBuf};
use crate::sample::Sample;
use crate::translate::Translate;

/// Make a mutable buffer into a write adapter that implements
/// [WriteBuf].
///
/// # Examples
///
/// ```rust
/// use rotary::{Buf as _, BufMut as _, ReadBuf as _, WriteBuf as _};
/// use rotary::io::{Read, Write};
///
/// let from = rotary::interleaved![[1.0f32, 2.0f32, 3.0f32, 4.0f32]; 2];
/// let to = rotary::interleaved![[0.0f32; 4]; 2];
/// let mut to = Write::new(to);
/// let mut from = Read::new(from.skip(2));
///
/// assert_eq!(to.remaining_mut(), 4);
/// to.copy(from);
/// assert_eq!(to.remaining_mut(), 2);
///
/// assert_eq! {
///     to.as_ref().as_slice(),
///     &[3.0, 3.0, 4.0, 4.0, 0.0, 0.0, 0.0, 0.0],
/// };
/// ```
pub struct Write<B> {
    buf: B,
    available: usize,
}

impl<B> Write<B>
where
    B: BufInfo,
{
    /// Construct a new write adapter.
    pub fn new(buf: B) -> Self {
        let available = buf.buf_info_frames();
        Self { buf, available }
    }

    /// Access the underlying buffer immutably.
    pub fn as_ref(&self) -> &B {
        &self.buf
    }

    /// Access the underlying buffer mutably.
    pub fn as_mut(&mut self) -> &mut B {
        &mut self.buf
    }
}

impl<B, T> WriteBuf<T> for Write<B>
where
    B: BufMut<T>,
    T: Sample,
{
    /// Remaining number of frames available.
    fn remaining_mut(&self) -> usize {
        self.available
    }

    /// Write to the underlying buffer.
    fn copy<I>(&mut self, mut buf: I)
    where
        I: ReadBuf + Buf<T>,
    {
        let len = usize::min(self.available, buf.buf_info_frames());
        crate::utils::copy(&buf, (&mut self.buf).tail(self.available));
        self.available = self.available.saturating_sub(len);
        buf.advance(len);
    }

    /// Write translated samples to the underlying buffer.
    fn translate<I, U>(&mut self, mut buf: I)
    where
        T: Translate<U>,
        I: ReadBuf + Buf<U>,
        U: Sample,
    {
        let len = usize::min(self.available, buf.remaining());
        crate::utils::translate(&buf, (&mut self.buf).tail(self.available));
        self.available = self.available.saturating_sub(len);
        buf.advance(len);
    }
}
