use crate::channel::Channel;
use crate::channel_mut::ChannelMut;
use std::fmt;
use std::ops;
use std::slice;

/// The mutable buffer of a single linear channel.
///
/// This doesn't provide direct access to the underlying buffer, but rather
/// allows us to copy data usinga  number of utility functions.
///
/// See [Channels::channel][crate::Channels::channel].
#[derive(PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LinearChannelMut<'a, T> {
    /// The underlying channel buffer.
    buf: &'a mut [T],
}

impl<'a, T> LinearChannelMut<'a, T> {
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
    pub fn new(buf: &'a mut [T]) -> Self {
        Self { buf }
    }
}

impl<'a, T> Channel for LinearChannelMut<'a, T> {
    type Sample = T;

    type Iter<'b>
    where
        T: 'b,
    = slice::Iter<'b, T>;

    fn frames(&self) -> usize {
        self.buf.len()
    }

    fn iter(&self) -> Self::Iter<'_> {
        self.buf.iter()
    }

    fn skip(self, n: usize) -> Self {
        Self {
            buf: self.buf.get_mut(n..).unwrap_or_default(),
        }
    }

    fn tail(self, n: usize) -> Self {
        let start = self.buf.len().saturating_sub(n);

        Self {
            buf: self.buf.get_mut(start..).unwrap_or_default(),
        }
    }

    fn limit(self, limit: usize) -> Self {
        Self {
            buf: self.buf.get_mut(..limit).unwrap_or_default(),
        }
    }

    fn chunk(self, n: usize, len: usize) -> Self {
        Self {
            buf: self.buf.get_mut(n..n + len).unwrap_or_default(),
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

impl<'a, T> ChannelMut for LinearChannelMut<'a, T> {
    type IterMut<'b>
    where
        T: 'b,
    = slice::IterMut<'b, T>;

    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        self.buf.iter_mut()
    }

    fn as_linear_mut(&mut self) -> Option<&mut [T]> {
        Some(self.buf)
    }
}

impl<T> fmt::Debug for LinearChannelMut<'_, T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.buf.iter()).finish()
    }
}

impl<'a, T> IntoIterator for LinearChannelMut<'a, T> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.buf.iter_mut()
    }
}

impl<T> ops::Index<usize> for LinearChannelMut<'_, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.buf[index]
    }
}
