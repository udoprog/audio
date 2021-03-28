//! Ranges that can be used for copying data.

/// Access the base range.
pub fn full() -> Full {
    Full(())
}

/// A range that can be used in combination with copying data.
pub trait Range {
    #[doc(hidden)]
    fn map_mut_linear<'a, T>(&self, buf: &'a mut [T]) -> &'a mut [T];

    #[doc(hidden)]
    fn map_iter_interleaved<'a, T: 'a, B, I, E>(&self, buf: B, iter: I, each: E)
    where
        B: Iterator<Item = &'a mut T>,
        I: IntoIterator<Item = T>,
        E: FnMut((&mut T, T));

    /// Construct a range with the given offset from the starting element.
    fn offset(self, offset: usize) -> Offset<Self>
    where
        Self: Sized + Range,
    {
        Offset { base: self, offset }
    }

    /// Construct a range which corresponds to the chunk with `len` and position
    /// `n`.
    ///
    /// Which is the range `n * len .. n * len + len`.
    fn chunk(self, n: usize, len: usize) -> Chunk<Self>
    where
        Self: Sized + Range,
    {
        Chunk { base: self, n, len }
    }
}

/// The full range.
pub struct Full(());

impl Range for Full {
    fn map_mut_linear<'a, T>(&self, buf: &'a mut [T]) -> &'a mut [T] {
        buf
    }

    fn map_iter_interleaved<'a, T: 'a, B, I, E>(&self, buf: B, iter: I, each: E)
    where
        B: Iterator<Item = &'a mut T>,
        I: IntoIterator<Item = T>,
        E: FnMut((&mut T, T)),
    {
        buf.zip(iter).for_each(each);
    }
}

/// A range that is an offset.
pub struct Offset<R> {
    #[allow(dead_code)]
    base: R,
    offset: usize,
}

impl Range for Offset<Full> {
    fn map_mut_linear<'a, T>(&self, buf: &'a mut [T]) -> &'a mut [T] {
        &mut buf[self.offset..]
    }

    fn map_iter_interleaved<'a, T: 'a, B, I, E>(&self, buf: B, iter: I, each: E)
    where
        B: Iterator<Item = &'a mut T>,
        I: IntoIterator<Item = T>,
        E: FnMut((&mut T, T)),
    {
        buf.skip(self.offset).zip(iter).for_each(each);
    }
}

/// A range that is chunk.
pub struct Chunk<R> {
    base: R,
    n: usize,
    len: usize,
}

impl Range for Chunk<Offset<Full>> {
    fn map_mut_linear<'a, T>(&self, buf: &'a mut [T]) -> &'a mut [T] {
        let buf = &mut buf[self.base.offset + self.n * self.len..];
        let len = usize::min(buf.len(), self.len);
        &mut buf[..len]
    }

    fn map_iter_interleaved<'a, T: 'a, B, I, E>(&self, buf: B, iter: I, each: E)
    where
        B: Iterator<Item = &'a mut T>,
        I: IntoIterator<Item = T>,
        E: FnMut((&mut T, T)),
    {
        buf.skip(self.base.offset + self.n * self.len)
            .take(self.len)
            .zip(iter)
            .for_each(each);
    }
}
