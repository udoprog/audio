//! Reading and writing sequentially from buffers.
//!
//! This is called buffered I/O, and allow buffers to support sequential reading
//! and writing to and from buffer.

/// A buffer that can keep track of how much has been read from it.
pub trait ReadBuf {
    /// Test if there are any remaining frames to read.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::ReadBuf as _;
    ///
    /// let mut buffer = rotary::wrap::interleaved(&[0, 1, 2, 3, 4, 5, 6, 7][..], 2);
    ///
    /// assert!(buffer.has_remaining());
    /// assert_eq!(buffer.remaining(), 4);
    /// buffer.advance(4);
    /// assert_eq!(buffer.remaining(), 0);
    /// ```
    fn has_remaining(&self) -> bool {
        self.remaining() > 0
    }

    /// Get the number of frames remaining that can be read from the buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::ReadBuf as _;
    ///
    /// let buffer = rotary::wrap::interleaved(&[0, 1, 2, 3, 4, 5, 6, 7][..], 2);
    ///
    /// assert_eq!(buffer.remaining(), 4);
    /// ```
    fn remaining(&self) -> usize;

    /// Advance the read number of frames by `n`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::ReadBuf as _;
    ///
    /// let mut buffer = rotary::wrap::interleaved(&[0, 1, 2, 3, 4, 5, 6, 7][..], 2);
    ///
    /// assert_eq!(buffer.remaining(), 4);
    /// buffer.advance(2);
    /// assert_eq!(buffer.remaining(), 2);
    /// ```
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
pub trait WriteBuf {
    /// Test if this buffer has remaining mutable frames.
    fn has_remaining_mut(&self) -> bool {
        self.remaining_mut() > 0
    }

    /// Remaining number of frames that can be written.
    fn remaining_mut(&self) -> usize;

    /// Advance the number of frames that has been written.
    fn advance_mut(&mut self, n: usize);
}

impl<B> WriteBuf for &'_ mut B
where
    B: WriteBuf,
{
    fn has_remaining_mut(&self) -> bool {
        (**self).has_remaining_mut()
    }

    fn remaining_mut(&self) -> usize {
        (**self).remaining_mut()
    }

    fn advance_mut(&mut self, n: usize) {
        (**self).advance_mut(n);
    }
}
