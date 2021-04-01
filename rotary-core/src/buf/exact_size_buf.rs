/// Trait used to describe a buffer that knows exactly how many frames it has
/// regardless of if it's sized or not.
///
/// # Examples
///
/// ```rust
/// use rotary::ExactSizeBuf;
///
/// fn test<T>(buf: T) where T: ExactSizeBuf {
///     assert_eq!(buf.frames(), 4);
/// }
///
/// test(rotary::interleaved![[0i16; 4]; 4]);
/// test(rotary::sequential![[0i16; 4]; 4]);
/// test(rotary::dynamic![[0i16; 4]; 4]);
/// test(rotary::wrap::interleaved([0i16; 16], 4));
/// test(rotary::wrap::sequential([0i16; 16], 4));
/// ```
pub trait ExactSizeBuf {
    /// The number of frames in a buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::ExactSizeBuf;
    ///
    /// fn test<T>(buf: T) where T: ExactSizeBuf {
    ///     assert_eq!(buf.frames(), 4);
    /// }
    ///
    /// test(rotary::interleaved![[0i16; 4]; 4]);
    /// test(rotary::sequential![[0i16; 4]; 4]);
    /// test(rotary::dynamic![[0i16; 4]; 4]);
    /// test(rotary::wrap::interleaved([0i16; 16], 4));
    /// test(rotary::wrap::sequential([0i16; 16], 4));
    /// ```
    fn frames(&self) -> usize;
}

impl<B> ExactSizeBuf for &B
where
    B: ?Sized + ExactSizeBuf,
{
    fn frames(&self) -> usize {
        (**self).frames()
    }
}

impl<B> ExactSizeBuf for &mut B
where
    B: ?Sized + ExactSizeBuf,
{
    fn frames(&self) -> usize {
        (**self).frames()
    }
}
