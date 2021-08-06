use crate::channel::Channel;
use crate::channel_mut::ChannelMut;
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
pub struct InterleavedChannelMut<'a, T> {
    buf: &'a mut [T],
    /// The number of channels in the interleaved buffer.
    channels: usize,
    /// The channel that is being accessed.
    channel: usize,
}

impl<'a, T> InterleavedChannelMut<'a, T> {
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
    pub fn new(buf: &'a mut [T], channels: usize, channel: usize) -> Self {
        Self {
            buf,
            channels,
            channel,
        }
    }
}

impl<'a, T> Channel<T> for InterleavedChannelMut<'a, T> {
    type Iter<'i>
    where
        T: 'i,
    = iter::StepBy<slice::Iter<'i, T>>;

    fn frames(&self) -> usize {
        self.buf.len() / self.channels
    }

    fn iter(&self) -> Self::Iter<'_> {
        let start = usize::min(self.channel, self.buf.len());
        self.buf[start..].iter().step_by(self.channels)
    }

    fn skip(self, n: usize) -> Self {
        Self {
            buf: self.buf.get_mut(n * self.channels..).unwrap_or_default(),
            channels: self.channels,
            channel: self.channel,
        }
    }

    fn tail(self, n: usize) -> Self {
        let start = self.buf.len().saturating_sub(n * self.channels);

        Self {
            buf: self.buf.get_mut(start..).unwrap_or_default(),
            channels: self.channels,
            channel: self.channel,
        }
    }

    fn limit(self, limit: usize) -> Self {
        Self {
            buf: self
                .buf
                .get_mut(..limit * self.channels)
                .unwrap_or_default(),
            channels: self.channels,
            channel: self.channel,
        }
    }

    fn chunk(self, n: usize, len: usize) -> Self {
        let len = len * self.channels;
        let n = n * len;

        Self {
            buf: self.buf.get_mut(n..n + len).unwrap_or_default(),
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

impl<'a, T> ChannelMut<T> for InterleavedChannelMut<'a, T> {
    type IterMut<'i>
    where
        T: 'i,
    = iter::StepBy<slice::IterMut<'i, T>>;

    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        let start = usize::min(self.channel, self.buf.len());
        self.buf[start..].iter_mut().step_by(self.channels)
    }

    fn as_linear_mut(&mut self) -> Option<&mut [T]> {
        None
    }
}

impl<T> fmt::Debug for InterleavedChannelMut<'_, T>
where
    T: Copy + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T> cmp::PartialEq for InterleavedChannelMut<'_, T>
where
    T: Copy + cmp::PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<T> cmp::Eq for InterleavedChannelMut<'_, T> where T: Copy + cmp::Eq {}

impl<T> cmp::PartialOrd for InterleavedChannelMut<'_, T>
where
    T: Copy + cmp::PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.iter().partial_cmp(other.iter())
    }
}

impl<T> cmp::Ord for InterleavedChannelMut<'_, T>
where
    T: Copy + cmp::Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.iter().cmp(other.iter())
    }
}

impl<T> hash::Hash for InterleavedChannelMut<'_, T>
where
    T: Copy + hash::Hash,
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

impl<'a, T> IntoIterator for InterleavedChannelMut<'a, T>
where
    T: Copy,
{
    type Item = &'a mut T;
    type IntoIter = iter::StepBy<slice::IterMut<'a, T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.buf[self.channel..].iter_mut().step_by(self.channels)
    }
}

impl<T> ops::Index<usize> for InterleavedChannelMut<'_, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.buf[self.channel + self.channels * index]
    }
}
