/// Trait used to govern sequential reading of an audio buffer.
///
/// This is the "in" part of "buffered I/O". It allows for buffers to govern
/// which slice of frames in them has been read so that operations can be
/// performed in multiple stages.
///
/// This can be accomplished manually using available buffer combinators such as
/// [Buf::tail][crate::Buf::tail]. But buffered I/O allows us to do this in a
/// much more structured fashion.
///
/// # Examples
///
/// ```rust
/// use audio::ReadBuf;
/// use audio::{io, wrap};
/// # fn send_data(buf: &mut [i16]) {}
///
/// // A simple mutable buffer we want to write to. Fits 2 channels with 64
/// // frames each.
/// let mut to = [0i16; 128];
///
/// // A buffer we want to read from. 2 channels with 512 frames each.
/// let from = audio::interleaved![[0i16; 512]; 2];
/// let mut from = io::Read::new(from);
///
/// let mut steps = 0;
///
/// while from.has_remaining() {
///     // Wrap the output buffer according to format so it can be written to
///     // correctly.
///     io::copy_remaining(&mut from, wrap::interleaved(&mut to[..], 2));
///
///     send_data(&mut to[..]);
///
///     steps += 1;
/// }
///
/// // We needed to write 8 times to copy our entire buffer.
/// assert_eq!(steps, 8);
/// ```
pub trait ReadBuf {
    /// Test if there are any remaining frames to read.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::ReadBuf;
    ///
    /// let mut buffer = audio::wrap::interleaved(&[0, 1, 2, 3, 4, 5, 6, 7][..], 2);
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
    /// use audio::ReadBuf;
    ///
    /// let buffer = audio::wrap::interleaved(&[0, 1, 2, 3, 4, 5, 6, 7][..], 2);
    ///
    /// assert_eq!(buffer.remaining(), 4);
    /// ```
    fn remaining(&self) -> usize;

    /// Advance the read number of frames by `n`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::ReadBuf;
    ///
    /// let mut buffer = audio::wrap::interleaved(&[0, 1, 2, 3, 4, 5, 6, 7][..], 2);
    ///
    /// assert_eq!(buffer.remaining(), 4);
    /// buffer.advance(2);
    /// assert_eq!(buffer.remaining(), 2);
    /// ```
    fn advance(&mut self, n: usize);
}

impl<B> ReadBuf for &mut B
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
