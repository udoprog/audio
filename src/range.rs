//! Ranges that can be used for copying data.

/// Access the base range.
pub fn base() -> Base {
    Base
}

/// A range that can be used in combination with copying data.
pub trait Range {
    #[doc(hidden)]
    fn copy_from_iter_linear<T, I>(&self, buf: &mut [T], iter: I)
    where
        I: IntoIterator<Item = T>;

    #[doc(hidden)]
    fn copy_from_iter_interleaved<T, I>(
        &self,
        channels: usize,
        channel: usize,
        buf: &mut [T],
        iter: I,
    ) where
        I: IntoIterator<Item = T>;

    /// Construct a range with the given offset.
    fn offset(self, offset: usize) -> Offset<Self>
    where
        Self: Sized + Range,
    {
        Offset { base: self, offset }
    }

    /// Construct a range with the given chunked setting.
    fn chunked(self, n: usize, len: usize) -> Chunked<Self>
    where
        Self: Sized + Range,
    {
        Chunked { base: self, n, len }
    }
}

/// The base range.
pub struct Base;

impl Range for Base {
    fn copy_from_iter_linear<T, I>(&self, buf: &mut [T], iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        for (o, f) in buf.iter_mut().zip(iter) {
            *o = f;
        }
    }

    fn copy_from_iter_interleaved<T, I>(
        &self,
        channels: usize,
        channel: usize,
        buf: &mut [T],
        iter: I,
    ) where
        I: IntoIterator<Item = T>,
    {
        for (o, f) in buf[channel..].iter_mut().step_by(channels).zip(iter) {
            *o = f;
        }
    }
}

/// A range that is an offset.
pub struct Offset<R> {
    #[allow(dead_code)]
    base: R,
    offset: usize,
}

impl Range for Offset<Base> {
    fn copy_from_iter_linear<T, I>(&self, buf: &mut [T], iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        let buf = &mut buf[self.offset..];

        for (o, f) in buf.iter_mut().zip(iter) {
            *o = f;
        }
    }

    fn copy_from_iter_interleaved<T, I>(
        &self,
        channels: usize,
        channel: usize,
        buf: &mut [T],
        iter: I,
    ) where
        I: IntoIterator<Item = T>,
    {
        let iter = buf[channel..]
            .iter_mut()
            .step_by(channels)
            .skip(self.offset)
            .zip(iter);

        for (o, f) in iter {
            *o = f;
        }
    }
}

/// A range that is chunked.
pub struct Chunked<R> {
    base: R,
    n: usize,
    len: usize,
}

impl Range for Chunked<Offset<Base>> {
    fn copy_from_iter_linear<T, I>(&self, buf: &mut [T], iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        let buf = &mut buf[self.base.offset + self.n * self.len..];
        let len = usize::min(buf.len(), self.len);
        let buf = &mut buf[..len];

        for (o, f) in buf.iter_mut().zip(iter) {
            *o = f;
        }
    }

    fn copy_from_iter_interleaved<T, I>(
        &self,
        channels: usize,
        channel: usize,
        buf: &mut [T],
        iter: I,
    ) where
        I: IntoIterator<Item = T>,
    {
        let iter = buf[channel..]
            .iter_mut()
            .step_by(channels)
            .skip(self.base.offset + self.n * self.len)
            .take(self.len)
            .zip(iter);

        for (o, f) in iter {
            *o = f;
        }
    }
}
