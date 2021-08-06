use crate::channel::Channel;
use std::cmp;
use std::fmt;
use std::hash;
use std::iter;
use std::ops;
use std::slice;

/// The buffer of a single interleaved channel.
///
/// This doesn't provide direct access to the underlying buffer, but rather
/// allows us to copy data usinga  number of utility functions.
///
/// See [Channels::channel][crate::Channels::channel].
#[derive(Clone, Copy)]
pub struct InterleavedChannel<'a, T> {
    buf: &'a [T],
    /// The number of channels in the interleaved buffer.
    channels: usize,
    /// The channel that is being accessed.
    channel: usize,
}

impl<'a, T> InterleavedChannel<'a, T> {
    /// Construct an interleaved channel buffer.
    ///
    /// The provided buffer must be the complete buffer, which includes *all*
    /// other channels. The provided `channels` argument is the total number of
    /// channels in this buffer, and `channel` indicates which specific channel
    /// this buffer belongs to.
    ///
    /// Note that this is typically not used directly, but instead through an
    /// abstraction which makes sure to provide the correct parameters.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio::InterleavedChannel;
    ///
    /// let buf = &[1, 2, 3, 4, 5, 6, 7, 8];
    /// let channel = InterleavedChannel::new(buf, 2, 1);
    ///
    /// assert_eq!(channel[1], 4);
    /// assert_eq!(channel[2], 6);
    /// ```
    pub fn new(buf: &'a [T], channels: usize, channel: usize) -> Self {
        Self {
            buf,
            channels,
            channel,
        }
    }
}

impl<'a, T> Channel<T> for InterleavedChannel<'a, T> {
    type Iter<'b>
    where
        T: 'b,
    = iter::StepBy<slice::Iter<'b, T>>;

    fn frames(&self) -> usize {
        self.buf.len() / self.channels
    }

    fn iter(&self) -> Self::Iter<'_> {
        let start = usize::min(self.channel, self.buf.len());
        self.buf[start..].iter().step_by(self.channels)
    }

    fn skip(self, n: usize) -> Self {
        Self {
            buf: self.buf.get(n * self.channels..).unwrap_or_default(),
            channels: self.channels,
            channel: self.channel,
        }
    }

    fn tail(self, n: usize) -> Self {
        let start = self.buf.len().saturating_sub(n * self.channels);

        Self {
            buf: self.buf.get(start..).unwrap_or_default(),
            channels: self.channels,
            channel: self.channel,
        }
    }

    fn limit(self, limit: usize) -> Self {
        Self {
            buf: self.buf.get(..limit * self.channels).unwrap_or_default(),
            channels: self.channels,
            channel: self.channel,
        }
    }

    fn chunk(self, n: usize, len: usize) -> Self {
        let len = len * self.channels;
        let n = n * len;

        Self {
            buf: self.buf.get(n..n + len).unwrap_or_default(),
            channels: self.channels,
            channel: self.channel,
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
        None
    }
}

impl<T> fmt::Debug for InterleavedChannel<'_, T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T> cmp::PartialEq for InterleavedChannel<'_, T>
where
    T: cmp::PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<T> cmp::Eq for InterleavedChannel<'_, T> where T: cmp::Eq {}

impl<T> cmp::PartialOrd for InterleavedChannel<'_, T>
where
    T: cmp::PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.iter().partial_cmp(other.iter())
    }
}

impl<T> cmp::Ord for InterleavedChannel<'_, T>
where
    T: cmp::Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.iter().cmp(other.iter())
    }
}

impl<T> hash::Hash for InterleavedChannel<'_, T>
where
    T: hash::Hash,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        for f in self.iter() {
            f.hash(state);
        }
    }
}

impl<'a, T> IntoIterator for InterleavedChannel<'a, T> {
    type Item = &'a T;
    type IntoIter = iter::StepBy<slice::Iter<'a, T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.buf[self.channel..].iter().step_by(self.channels)
    }
}

impl<T> ops::Index<usize> for InterleavedChannel<'_, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.buf[self.channel + self.channels * index]
    }
}
