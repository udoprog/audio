/// Trait used to govern sequential writing to an audio buffer.
///
/// This is the "out" part of "buffered I/O". It allows for buffers to govern
/// which slice of frames in them has been read so that operations can be
/// performed in multiple stages.
///
/// This can be accomplished manually using available buffer combinators such as
/// [Buf::tail][crate::Buf::tail]. But buffered I/O allows us to do this in a
/// much more structured fashion.
///
/// # Examples
///
/// ```
/// use audio::WriteBuf;
/// use audio::{io, wrap};
/// # fn recv_data(buf: &mut [i16]) {}
///
/// // A simple buffer we want to read from to. Fits 2 channels with 64
/// // frames each.
/// let mut from = [0i16; 128];
///
/// // A buffer we want to write to. 2 channels with 512 frames each.
/// let to = audio::interleaved![[0i16; 512]; 2];
/// let mut to = io::Write::new(to);
///
/// let mut steps = 0;
///
/// while to.has_remaining_mut() {
///     // Fill the buffer with something interesting.
///     recv_data(&mut from[..]);
///
///     // Wrap the filled buffer according to format so it can be written to
///     // correctly.
///     io::copy_remaining(wrap::interleaved(&mut from[..], 2), &mut to);
///
///     steps += 1;
/// }
///
/// // We needed to write 8 times to fill our entire buffer.
/// assert_eq!(steps, 8);
/// ```
pub trait WriteBuf {
    /// Test if this buffer has remaining mutable frames that can be written.
    ///
    /// # Examples
    ///
    /// ```
    /// use audio::WriteBuf;
    ///
    /// let mut buf = [0, 1, 2, 3, 4, 5, 6, 7];
    /// let mut buf = audio::wrap::interleaved(&mut buf[..], 2);
    ///
    /// assert!(buf.has_remaining_mut());
    /// assert_eq!(buf.remaining_mut(), 4);
    /// buf.advance_mut(4);
    /// assert_eq!(buf.remaining_mut(), 0);
    /// ```
    fn has_remaining_mut(&self) -> bool {
        self.remaining_mut() > 0
    }

    /// Remaining number of frames that can be written.
    ///
    /// # Examples
    ///
    /// ```
    /// use audio::WriteBuf;
    ///
    /// let mut buf = [0, 1, 2, 3, 4, 5, 6, 7];
    /// let buf = audio::wrap::interleaved(&mut buf[..], 2);
    ///
    /// assert_eq!(buf.remaining_mut(), 4);
    /// ```
    fn remaining_mut(&self) -> usize;

    /// Advance the number of frames that have been written.
    ///
    /// # Examples
    ///
    /// ```
    /// use audio::WriteBuf;
    ///
    /// let mut buf = [0, 1, 2, 3, 4, 5, 6, 7];
    /// let mut buf = audio::wrap::interleaved(&mut buf[..], 2);
    ///
    /// assert_eq!(buf.remaining_mut(), 4);
    /// buf.advance_mut(2);
    /// assert_eq!(buf.remaining_mut(), 2);
    /// ```
    fn advance_mut(&mut self, n: usize);
}

impl<B> WriteBuf for &mut B
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
