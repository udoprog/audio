use crate::channel::Channel;
use std::fmt;
use std::ops;
use std::slice;

/// The buffer of a single linear channel.
///
/// This doesn't provide direct access to the underlying buffer, but rather
/// allows us to copy data usinga  number of utility functions.
///
/// See [Channels::channel][crate::Channels::channel].
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LinearChannel<'a, T> {
    /// The underlying channel buffer.
    buf: &'a [T],
}

impl<'a, T> LinearChannel<'a, T> {
    /// Construct a linear channel buffer.
    ///
    /// The buffer provided as-is constitutes the frames of the channel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::LinearChannel;
    ///
    /// let buf = &mut [1, 3, 5, 7];
    /// let channel = LinearChannel::new(buf);
    ///
    /// assert_eq!(channel[1], 3);
    /// assert_eq!(channel[2], 5);
    /// ```
    pub fn new(buf: &'a [T]) -> Self {
        Self { buf }
    }
}

impl<'a, T> Channel for LinearChannel<'a, T> {
    type Sample = T;

    type Iter<'i>
    where
        T: 'i,
    = slice::Iter<'i, T>;

    fn frames(&self) -> usize {
        self.buf.len()
    }

    fn iter(&self) -> Self::Iter<'a> {
        self.buf.iter()
    }

    fn skip(self, n: usize) -> Self {
        Self {
            buf: self.buf.get(n..).unwrap_or_default(),
        }
    }

    fn tail(self, n: usize) -> Self {
        let start = self.buf.len().saturating_sub(n);

        Self {
            buf: self.buf.get(start..).unwrap_or_default(),
        }
    }

    fn limit(self, limit: usize) -> Self {
        Self {
            buf: self.buf.get(..limit).unwrap_or_default(),
        }
    }

    fn chunk(self, n: usize, len: usize) -> Self {
        Self {
            buf: self.buf.get(n..n + len).unwrap_or_default(),
        }
    }

    fn chunks(&self, chunk: usize) -> usize {
        let len = self.frames();

        if len % chunk == 0 {
            len / chunk
        } else {
            len / chunk + 1
        }
    }

    fn as_linear(&self) -> Option<&[T]> {
        Some(self.buf)
    }
}

impl<T> fmt::Debug for LinearChannel<'_, T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.buf).finish()
    }
}

impl<'a, T> IntoIterator for LinearChannel<'a, T> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.buf.iter()
    }
}

impl<T> ops::Index<usize> for LinearChannel<'_, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.buf[index]
    }
}
