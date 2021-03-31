//! A channel buffer as created through [Buf::channel][crate::Buf::channel] or
//! [BufMut::channel_mut][crate::BufMut::channel_mut].

use crate::buf::ChannelKind;
use crate::translate::Translate;
use std::ops;

mod iter;
pub use self::iter::{Iter, IterMut};

/// The buffer of a single channel.
///
/// This doesn't provide direct access to the underlying buffer, but rather
/// allows us to copy data usinga  number of utility functions.
#[derive(Debug, Clone, Copy)]
pub struct Channel<'a, T> {
    pub(crate) buf: &'a [T],
    pub(crate) kind: ChannelKind,
}

impl<'a, T> Channel<'a, T> {
    /// Construct a linear buffer.
    pub fn linear(buf: &'a [T]) -> Self {
        Self {
            buf,
            kind: ChannelKind::Linear,
        }
    }

    /// Construct an interleaved buffer.
    pub fn interleaved(buf: &'a [T], channels: usize, channel: usize) -> Self {
        Self {
            buf,
            kind: ChannelKind::Interleaved { channels, channel },
        }
    }

    /// Access the number of frames on the current channel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::Buf;
    ///
    /// fn test(buf: &dyn Buf<f32>) {
    ///     let left = buf.channel(0);
    ///     let right = buf.channel(1);
    ///
    ///     assert_eq!(left.frames(), 16);
    ///     assert_eq!(right.frames(), 16);
    /// }
    ///
    /// test(&rotary::dynamic![[0.0; 16]; 2]);
    /// test(&rotary::sequential![[0.0; 16]; 2]);
    /// test(&rotary::interleaved![[0.0; 16]; 2]);
    /// ```
    pub fn frames(&self) -> usize {
        match self.kind {
            ChannelKind::Linear => self.buf.len(),
            ChannelKind::Interleaved { channels, .. } => self.buf.len() / channels,
        }
    }

    /// Construct an iterator over the channel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::{Buf as _, BufMut as _};
    ///
    /// let mut left = rotary::interleaved![[0.0f32; 4]; 2];
    /// let mut right = rotary::dynamic![[0.0f32; 4]; 2];
    ///
    /// for (l, r) in left.channel_mut(0).iter_mut().zip(right.channel_mut(0)) {
    ///     *l = 1.0;
    ///     *r = 1.0;
    /// }
    ///
    /// assert!(left.channel(0).iter().eq(right.channel(0).iter()));
    ///
    /// assert_eq!(left.as_slice(), &[1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0]);
    /// assert_eq!(&right[0], &[1.0, 1.0, 1.0, 1.0]);
    /// assert_eq!(&right[1], &[0.0, 0.0, 0.0, 0.0]);
    /// ```
    pub fn iter(self) -> Iter<'a, T> {
        match self.kind {
            ChannelKind::Linear => Iter::new(self.buf, 1),
            ChannelKind::Interleaved { channels, channel } => {
                let start = usize::min(channel, self.buf.len());
                Iter::new(&self.buf[start..], channels)
            }
        }
    }

    /// Construct a new mutable channel that has a lifetime of the current
    /// instance.
    pub fn as_ref(&self) -> Channel<'_, T> {
        Channel {
            buf: self.buf,
            kind: self.kind,
        }
    }

    /// Construct a channel buffer where the first `n` frames are skipped.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::{Buf as _, BufMut as _};
    ///
    /// let mut from = rotary::interleaved![[0.0f32; 4]; 2];
    /// *from.frame_mut(0, 2).unwrap() = 1.0;
    /// *from.frame_mut(0, 3).unwrap() = 1.0;
    ///
    /// let mut to = rotary::interleaved![[0.0f32; 4]; 2];
    ///
    /// to.channel_mut(0).copy_from(from.channel(0).skip(2));
    /// assert_eq!(to.as_slice(), &[1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    /// ```
    pub fn skip(self, n: usize) -> Self {
        let Self { buf, kind } = self;

        match kind {
            ChannelKind::Linear => Self {
                buf: buf.get(n..).unwrap_or_default(),
                kind,
            },
            ChannelKind::Interleaved { channels, .. } => Self {
                buf: buf.get(n * channels..).unwrap_or_default(),
                kind,
            },
        }
    }

    /// Construct a channel buffer where the last `n` frames are included.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::{Buf as _, BufMut as _};
    ///
    /// let from = rotary::interleaved![[1.0f32; 4]; 2];
    /// let mut to = rotary::interleaved![[0.0f32; 4]; 2];
    ///
    /// to.channel_mut(0).as_mut().tail(2).copy_from(from.channel(0));
    /// assert_eq!(to.as_slice(), &[0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0]);
    /// ```
    pub fn tail(self, n: usize) -> Self {
        let Self { buf, kind } = self;

        match kind {
            ChannelKind::Linear => {
                let start = buf.len().saturating_sub(n);

                Self {
                    buf: buf.get(start..).unwrap_or_default(),
                    kind,
                }
            }
            ChannelKind::Interleaved { channels, .. } => {
                let start = buf.len().saturating_sub(n * channels);

                Self {
                    buf: buf.get(start..).unwrap_or_default(),
                    kind,
                }
            }
        }
    }

    /// Limit the channel bufferto `limit` number of frames.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::{Buf as _, BufMut as _};
    ///
    /// let from = rotary::interleaved![[1.0f32; 4]; 2];
    /// let mut to = rotary::interleaved![[0.0f32; 4]; 2];
    ///
    /// to.channel_mut(0).copy_from(from.channel(0).limit(2));
    /// assert_eq!(to.as_slice(), &[1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    /// ```
    pub fn limit(self, limit: usize) -> Self {
        let Self { buf, kind } = self;

        match kind {
            ChannelKind::Linear => Channel {
                buf: buf.get(..limit).unwrap_or_default(),
                kind,
            },
            ChannelKind::Interleaved { channels, .. } => Channel {
                buf: buf.get(..limit * channels).unwrap_or_default(),
                kind,
            },
        }
    }

    /// Construct a range of frames corresponds to the chunk with `len` and
    /// position `n`.
    ///
    /// Which is the range `n * len .. n * len + len`.
    pub fn chunk(self, n: usize, len: usize) -> Self {
        let Self { buf, kind } = self;

        match kind {
            ChannelKind::Linear => Channel {
                buf: buf.get(n..n + len).unwrap_or_default(),
                kind,
            },
            ChannelKind::Interleaved { channels, .. } => {
                let len = len * channels;
                let n = n * len;

                Channel {
                    buf: buf.get(n..n + len).unwrap_or_default(),
                    kind,
                }
            }
        }
    }

    /// How many chunks of the given size can you divide buf into.
    ///
    /// This includes one extra chunk even if the chunk doesn't divide the frame
    /// length evenly.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::Buf;
    ///
    /// fn test(buf: &dyn Buf<f32>) {
    ///     let left = buf.channel(0);
    ///     let right = buf.channel(1);
    ///
    ///     assert_eq!(left.chunks(4), 4);
    ///     assert_eq!(right.chunks(4), 4);
    ///
    ///     assert_eq!(left.chunks(6), 3);
    ///     assert_eq!(right.chunks(6), 3);
    /// }
    ///
    /// test(&rotary::dynamic![[0.0; 16]; 2]);
    /// test(&rotary::sequential![[0.0; 16]; 2]);
    /// test(&rotary::interleaved![[0.0; 16]; 2]);
    /// ```
    pub fn chunks(&self, chunk: usize) -> usize {
        let len = self.frames();

        if len % chunk == 0 {
            len / chunk
        } else {
            len / chunk + 1
        }
    }

    /// Copy into the given slice of output.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::Buf;
    ///
    /// fn test(buf: &dyn Buf<f32>) {
    ///     let channel = buf.channel(0);
    ///
    ///     let mut buf = vec![0.0; 16];
    ///     channel.copy_into_slice(&mut buf[..]);
    ///
    ///     assert!(buf.iter().all(|f| *f == 1.0));
    /// }
    ///
    /// test(&rotary::dynamic![[1.0; 16]; 2]);
    /// test(&rotary::sequential![[1.0; 16]; 2]);
    /// test(&rotary::interleaved![[1.0; 16]; 2]);
    /// ```
    pub fn copy_into_slice(&self, out: &mut [T])
    where
        T: Copy,
    {
        match self.kind {
            ChannelKind::Linear => {
                let end = usize::min(out.len(), self.buf.len());
                out[..end].copy_from_slice(&self.buf[..end]);
            }
            ChannelKind::Interleaved { channels, channel } => {
                for (o, f) in out
                    .iter_mut()
                    .zip(self.buf[channel..].iter().step_by(channels))
                {
                    *o = *f;
                }
            }
        }
    }

    /// Copy into the given iterator.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::Buf;
    ///
    /// fn test(buf: &dyn Buf<f32>) {
    ///     let channel = buf.channel(0);
    ///
    ///     let mut buf = vec![0.0; 16];
    ///
    ///     // Copy into every other position in `buf`.
    ///     channel.copy_into_iter(buf.iter_mut().step_by(2));
    ///
    ///     for (n, f) in buf.into_iter().enumerate() {
    ///         if n % 2 == 0 {
    ///             assert_eq!(f, 1.0);
    ///         } else {
    ///             assert_eq!(f, 0.0);
    ///         }
    ///     }
    /// }
    ///
    /// test(&rotary::dynamic![[1.0; 16]; 2]);
    /// test(&rotary::sequential![[1.0; 16]; 2]);
    /// test(&rotary::interleaved![[1.0; 16]; 2]);
    /// ```
    pub fn copy_into_iter<'out, I>(&self, iter: I)
    where
        I: IntoIterator<Item = &'out mut T>,
        T: 'out + Copy,
    {
        match self.kind {
            ChannelKind::Linear => {
                for (o, f) in iter.into_iter().zip(self.buf) {
                    *o = *f;
                }
            }
            ChannelKind::Interleaved { channels, channel } => {
                for (o, f) in iter
                    .into_iter()
                    .zip(self.buf[channel..].iter().step_by(channels))
                {
                    *o = *f;
                }
            }
        }
    }
}

impl<'a, T> IntoIterator for Channel<'a, T>
where
    T: Copy,
{
    type Item = T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a Channel<'_, T>
where
    T: Copy,
{
    type Item = T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_ref().iter()
    }
}

impl<T> ops::Index<usize> for Channel<'_, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        match self.kind {
            ChannelKind::Linear => &self.buf[index],
            ChannelKind::Interleaved { channels, channel } => &self.buf[channel + channels * index],
        }
    }
}

/// The mutable buffer of a single channel.
///
/// This doesn't provide direct access to the underlying buffer, but rather
/// allows us to copy data usinga  number of utility functions.
#[derive(Debug)]
pub struct ChannelMut<'a, T> {
    pub(crate) buf: &'a mut [T],
    pub(crate) kind: ChannelKind,
}

impl<'a, T> ChannelMut<'a, T> {
    /// Construct a linear buffer.
    pub fn linear(buf: &'a mut [T]) -> Self {
        Self {
            buf,
            kind: ChannelKind::Linear,
        }
    }

    /// Construct an interleaved buffer.
    pub fn interleaved(buf: &'a mut [T], channels: usize, channel: usize) -> Self {
        Self {
            buf,
            kind: ChannelKind::Interleaved { channels, channel },
        }
    }

    /// Construct a new mutable channel that has a lifetime of the current
    /// instance.
    pub fn as_mut(&mut self) -> ChannelMut<'_, T> {
        ChannelMut {
            buf: self.buf,
            kind: self.kind,
        }
    }

    /// The number of frames in the buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::BufMut;
    ///
    /// fn test(buf: &dyn BufMut<f32>) {
    ///     let left = buf.channel(0);
    ///     let right = buf.channel(1);
    ///
    ///     assert_eq!(left.frames(), 16);
    ///     assert_eq!(right.frames(), 16);
    /// }
    ///
    /// test(&rotary::dynamic![[0.0; 16]; 2]);
    /// test(&rotary::sequential![[0.0; 16]; 2]);
    /// test(&rotary::interleaved![[0.0; 16]; 2]);
    /// ```
    pub fn frames(&self) -> usize {
        match self.kind {
            ChannelKind::Linear => self.buf.len(),
            ChannelKind::Interleaved { channels, .. } => self.buf.len() / channels,
        }
    }

    /// The number of chunks that can fit with the given size.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::BufMut;
    ///
    /// fn test(buf: &dyn BufMut<f32>) {
    ///     let left = buf.channel(0);
    ///     let right = buf.channel(1);
    ///
    ///     assert_eq!(left.chunks(4), 4);
    ///     assert_eq!(right.chunks(4), 4);
    ///
    ///     assert_eq!(left.chunks(6), 3);
    ///     assert_eq!(right.chunks(6), 3);
    /// }
    ///
    /// test(&rotary::dynamic![[0.0; 16]; 2]);
    /// test(&rotary::sequential![[0.0; 16]; 2]);
    /// test(&rotary::interleaved![[0.0; 16]; 2]);
    /// ```
    pub fn chunks(&self, chunk: usize) -> usize {
        let len = self.frames();

        if len % chunk == 0 {
            len / chunk
        } else {
            len / chunk + 1
        }
    }

    /// Construct an iterator over the channel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::{Buf as _, BufMut as _};
    ///
    /// let mut left = rotary::interleaved![[0.0f32; 4]; 2];
    /// let mut right = rotary::dynamic![[0.0f32; 4]; 2];
    ///
    /// for (l, r) in left.channel_mut(0).iter_mut().zip(right.channel_mut(0)) {
    ///     *l = 1.0;
    ///     *r = 1.0;
    /// }
    ///
    /// assert!(left.channel(0).iter().eq(right.channel(0).iter()));
    ///
    /// assert_eq!(left.as_slice(), &[1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0]);
    /// assert_eq!(&right[0], &[1.0, 1.0, 1.0, 1.0]);
    /// assert_eq!(&right[1], &[0.0, 0.0, 0.0, 0.0]);
    /// ```
    pub fn iter(self) -> Iter<'a, T> {
        match self.kind {
            ChannelKind::Linear => Iter::new(self.buf, 1),
            ChannelKind::Interleaved { channels, channel } => {
                let start = usize::min(channel, self.buf.len());
                Iter::new(&self.buf[start..], channels)
            }
        }
    }

    /// Construct a mutable iterator over the channel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::{Buf as _, BufMut as _};
    ///
    /// let mut left = rotary::interleaved![[0.0f32; 4]; 2];
    /// let mut right = rotary::dynamic![[0.0f32; 4]; 2];
    ///
    /// for (l, r) in left.channel_mut(0).iter_mut().zip(right.channel_mut(0)) {
    ///     *l = 1.0;
    ///     *r = 1.0;
    /// }
    ///
    /// assert!(left.channel(0).iter().eq(right.channel(0).iter()));
    ///
    /// assert_eq!(left.as_slice(), &[1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0]);
    /// assert_eq!(&right[0], &[1.0, 1.0, 1.0, 1.0]);
    /// assert_eq!(&right[1], &[0.0, 0.0, 0.0, 0.0]);
    /// ```
    pub fn iter_mut(self) -> IterMut<'a, T> {
        match self.kind {
            ChannelKind::Linear => IterMut::new(self.buf, 1),
            ChannelKind::Interleaved { channels, channel } => {
                let start = usize::min(channel, self.buf.len());
                IterMut::new(&mut self.buf[start..], channels)
            }
        }
    }

    /// Construct a channel buffer where the first `n` frames are skipped.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::{Buf as _, BufMut as _};
    ///
    /// let mut buffer = rotary::Interleaved::with_topology(2, 4);
    ///
    /// buffer.channel_mut(0).skip(2).copy_from_slice(&[1.0, 1.0]);
    ///
    /// assert_eq!(buffer.as_slice(), &[0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0])
    /// ```
    pub fn skip(self, n: usize) -> Self {
        let Self { buf, kind } = self;

        match kind {
            ChannelKind::Linear => Self {
                buf: buf.get_mut(n..).unwrap_or_default(),
                kind,
            },
            ChannelKind::Interleaved { channels, .. } => Self {
                buf: buf.get_mut(n * channels..).unwrap_or_default(),
                kind,
            },
        }
    }

    /// Construct a channel buffer where the last `n` frames are included.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::{Buf as _, BufMut as _};
    ///
    /// let from = rotary::interleaved![[1.0f32; 4]; 2];
    /// let mut to = rotary::interleaved![[0.0f32; 4]; 2];
    ///
    /// to.channel_mut(0).as_mut().tail(2).copy_from(from.channel(0));
    /// assert_eq!(to.as_slice(), &[0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0]);
    /// ```
    pub fn tail(self, n: usize) -> Self {
        let Self { buf, kind } = self;

        match kind {
            ChannelKind::Linear => {
                let start = buf.len().saturating_sub(n);

                Self {
                    buf: buf.get_mut(start..).unwrap_or_default(),
                    kind,
                }
            }
            ChannelKind::Interleaved { channels, .. } => {
                let start = buf.len().saturating_sub(n * channels);

                Self {
                    buf: buf.get_mut(start..).unwrap_or_default(),
                    kind,
                }
            }
        }
    }

    /// Limit the channel bufferto `limit` number of frames.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::{Buf as _, BufMut as _};
    ///
    /// let from = rotary::interleaved![[1.0f32; 4]; 2];
    /// let mut to = rotary::interleaved![[0.0f32; 4]; 2];
    ///
    /// to.channel_mut(0).limit(2).copy_from(from.channel(0));
    /// assert_eq!(to.as_slice(), &[1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    /// ```
    pub fn limit(self, limit: usize) -> Self {
        let Self { buf, kind } = self;

        match kind {
            ChannelKind::Linear => Self {
                buf: buf.get_mut(..limit).unwrap_or_default(),
                kind,
            },
            ChannelKind::Interleaved { channels, .. } => Self {
                buf: buf.get_mut(..limit * channels).unwrap_or_default(),
                kind,
            },
        }
    }

    /// Construct a range of frames corresponds to the chunk with `len` and
    /// position `n`.
    ///
    /// Which is the range `n * len .. n * len + len`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::{Buf as _, BufMut as _};
    ///
    /// let from = rotary::interleaved![[1.0f32; 4]; 2];
    /// let mut to = rotary::interleaved![[0.0f32; 4]; 2];
    ///
    /// to.channel_mut(0).chunk(1, 2).copy_from(from.channel(0));
    /// assert_eq!(to.as_slice(), &[0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0]);
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::Buf;
    ///
    /// fn test(buf: &dyn Buf<f32>) {
    ///     let channel = buf.channel(0);
    ///
    ///     let mut buf = vec![0.0; 4];
    ///     channel.chunk(3, 4).copy_into_slice(&mut buf[..]);
    ///
    ///     assert!(buf.iter().all(|f| *f == 1.0));
    /// }
    ///
    /// test(&rotary::dynamic![[1.0; 16]; 2]);
    /// test(&rotary::sequential![[1.0; 16]; 2]);
    /// test(&rotary::interleaved![[1.0; 16]; 2]);
    /// ```
    pub fn chunk(self, n: usize, len: usize) -> Self {
        let Self { buf, kind } = self;

        match kind {
            ChannelKind::Linear => Self {
                buf: buf.get_mut(n..n + len).unwrap_or_default(),
                kind,
            },
            ChannelKind::Interleaved { channels, .. } => {
                let len = len * channels;
                let n = n * len;

                Self {
                    buf: buf.get_mut(n..n + len).unwrap_or_default(),
                    kind,
                }
            }
        }
    }

    /// Copy from the given slice.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::BufMut;
    ///
    /// fn test(buf: &mut dyn BufMut<f32>) {
    ///     buf.channel_mut(0).copy_from_slice(&[1.0; 4][..]);
    ///
    ///     let mut out = vec![0.0; 8];
    ///     buf.channel(0).copy_into_slice(&mut out);
    ///
    ///     assert_eq!(out, vec![1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0]);
    /// }
    ///
    /// test(&mut rotary::dynamic![[0.0; 8]; 2]);
    /// test(&mut rotary::sequential![[0.0; 8]; 2]);
    /// test(&mut rotary::interleaved![[0.0; 8]; 2]);
    /// ```
    pub fn copy_from_slice(&mut self, buf: &[T])
    where
        T: Copy,
    {
        match self.kind {
            ChannelKind::Linear => {
                let len = usize::min(self.buf.len(), buf.len());
                self.buf[..len].copy_from_slice(&buf[..len]);
            }
            ChannelKind::Interleaved { channels, channel } => {
                for (o, f) in self.buf[channel..].iter_mut().step_by(channels).zip(buf) {
                    *o = *f;
                }
            }
        }
    }

    /// Copy a chunked destination from an iterator.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::BufMut;
    ///
    /// fn test(buf: &mut dyn BufMut<f32>) {
    ///     buf.channel_mut(0).skip(2).copy_from_iter(vec![1.0; 4]);
    ///
    ///     let mut out = vec![0.0; 8];
    ///     buf.channel(0).copy_into_slice(&mut out);
    ///
    ///     assert_eq!(out, vec![0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0]);
    /// }
    ///
    /// test(&mut rotary::dynamic![[0.0; 8]; 2]);
    /// test(&mut rotary::sequential![[0.0; 8]; 2]);
    /// test(&mut rotary::interleaved![[0.0; 8]; 2]);
    /// ```
    ///
    /// ```rust
    /// use rotary::BufMut;
    ///
    /// fn test(buf: &mut dyn BufMut<f32>) {
    ///     buf.channel_mut(0).skip(2).chunk(0, 2).copy_from_iter(vec![1.0; 4]);
    ///
    ///     let mut out = vec![0.0; 8];
    ///     buf.channel(0).copy_into_slice(&mut out);
    ///
    ///     assert_eq!(out, vec![0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0]);
    /// }
    ///
    /// test(&mut rotary::dynamic![[0.0; 8]; 2]);
    /// test(&mut rotary::sequential![[0.0; 8]; 2]);
    /// test(&mut rotary::interleaved![[0.0; 8]; 2]);
    /// ```
    pub fn copy_from_iter<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        match self.kind {
            ChannelKind::Linear => {
                for (o, f) in self.buf.iter_mut().zip(iter) {
                    *o = f;
                }
            }
            ChannelKind::Interleaved { channels, channel } => {
                let buf = self.buf[channel..].iter_mut().step_by(channels);

                for (o, f) in buf.zip(iter) {
                    *o = f;
                }
            }
        }
    }

    /// Copy one channel from another.
    ///
    /// See [utils::copy][crate::utils::copy] if you want to copy an entire
    /// buffer into another.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::{Buf as _, BufMut as _};
    ///
    /// let from = rotary::dynamic![[1.0f32; 4]; 2];
    /// let mut to = rotary::interleaved![[0.0f32; 4]; 3];
    ///
    /// to.channel_mut(0).copy_from(from.channel(1));
    /// assert_eq!(to.as_slice(), &[1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
    /// ```
    pub fn copy_from(&mut self, from: Channel<'_, T>)
    where
        T: Copy,
    {
        match (self.kind, from.kind) {
            (ChannelKind::Linear, ChannelKind::Linear) => {
                self.buf.copy_from_slice(&from.buf[..]);
            }
            _ => {
                for (o, f) in self.as_mut().iter_mut().zip(from) {
                    *o = f;
                }
            }
        }
    }

    /// Translate one channel from another.
    ///
    /// This will copy the channel while translating each sample according to
    /// its [Translate] implementation.
    ///
    /// This is used for converting one type of sample to another.
    ///
    /// See [utils::translate][crate::utils::translate] if you want to translate
    /// an entire buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::{Buf as _, BufMut as _};
    ///
    /// let from = rotary::dynamic![[u16::MAX; 4]; 2];
    /// let mut to = rotary::interleaved![[0.0f32; 4]; 3];
    ///
    /// to.channel_mut(0).translate_from(from.channel(1));
    /// assert_eq!(to.as_slice(), &[1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
    /// ```
    pub fn translate_from<U>(&mut self, from: Channel<'_, U>)
    where
        U: Copy,
        T: Translate<U>,
    {
        for (o, f) in self.as_mut().iter_mut().zip(from) {
            *o = T::translate(f);
        }
    }
}

impl<'a, T> IntoIterator for ChannelMut<'a, T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, T> IntoIterator for &'a mut ChannelMut<'_, T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_mut().iter_mut()
    }
}

impl<T> ops::Index<usize> for ChannelMut<'_, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        match self.kind {
            ChannelKind::Linear => &self.buf[index],
            ChannelKind::Interleaved { channels, channel } => &self.buf[channel + channels * index],
        }
    }
}

/// Get a mutable reference to the frame at the given index.
///
/// # Panics
///
/// Panics if the given frame is out of bounds for this channel.
///
/// See [frames][Self::frames].
///
/// # Examples
///
/// ```rust
/// use rotary::BufMut;
///
/// fn test(buf: &mut dyn BufMut<f32>) {
///     buf.channel_mut(0)[1] = 1.0;
///     buf.channel_mut(0)[7] = 1.0;
///
///     let mut out = vec![0.0; 8];
///     buf.channel(0).copy_into_slice(&mut out);
///
///     assert_eq!(out, vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0]);
/// }
///
/// test(&mut rotary::dynamic![[0.0; 8]; 2]);
/// test(&mut rotary::sequential![[0.0; 8]; 2]);
/// test(&mut rotary::interleaved![[0.0; 8]; 2]);
/// ```
impl<T> ops::IndexMut<usize> for ChannelMut<'_, T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match self.kind {
            ChannelKind::Linear => &mut self.buf[index],
            ChannelKind::Interleaved { channels, channel } => {
                &mut self.buf[channel + channels * index]
            }
        }
    }
}