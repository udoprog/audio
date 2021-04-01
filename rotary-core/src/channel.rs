//! A channel buffer as created through [Channels::channel][crate::Channels::channel] or
//! [ChannelsMut::channel_mut][crate::ChannelsMut::channel_mut].

use crate::translate::Translate;
use std::ops;

mod iter;
pub use self::iter::{Iter, IterMut};

/// Used to determine how a buffer is indexed.
#[derive(Debug, Clone, Copy)]
enum Kind {
    /// Returned channel buffer is indexed in a linear manner.
    Linear,
    /// Returned channel buffer is indexed in an interleaved manner.
    Interleaved {
        /// The number of channels in the interleaved buffer.
        channels: usize,
        /// The channel that is being accessed.
        channel: usize,
    },
}

/// The buffer of a single channel.
///
/// This doesn't provide direct access to the underlying buffer, but rather
/// allows us to copy data usinga  number of utility functions.
#[derive(Debug, Clone, Copy)]
pub struct Channel<'a, T> {
    buf: &'a [T],
    kind: Kind,
}

impl<'a, T> Channel<'a, T> {
    /// Construct a linear channel buffer.
    ///
    /// The buffer provided as-is constitutes the frames of the channel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::Channel;
    ///
    /// let buf = &mut [1, 3, 5, 7];
    /// let channel = Channel::linear(buf);
    ///
    /// assert_eq!(channel[1], 3);
    /// assert_eq!(channel[2], 5);
    /// ```
    pub fn linear(buf: &'a [T]) -> Self {
        Self {
            buf,
            kind: Kind::Linear,
        }
    }

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
    /// use rotary::Channel;
    ///
    /// let buf = &[1, 2, 3, 4, 5, 6, 7, 8];
    /// let channel = Channel::interleaved(buf, 2, 1);
    ///
    /// assert_eq!(channel[1], 4);
    /// assert_eq!(channel[2], 6);
    /// ```
    pub fn interleaved(buf: &'a [T], channels: usize, channel: usize) -> Self {
        Self {
            buf,
            kind: Kind::Interleaved { channels, channel },
        }
    }

    /// Access the number of frames on the current channel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::Channels;
    ///
    /// fn test(buf: &dyn Channels<f32>) {
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
            Kind::Linear => self.buf.len(),
            Kind::Interleaved { channels, .. } => self.buf.len() / channels,
        }
    }

    /// Construct an iterator over the channel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::{Channels as _, ChannelsMut as _};
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
            Kind::Linear => Iter::new(self.buf, 1),
            Kind::Interleaved { channels, channel } => {
                let start = usize::min(channel, self.buf.len());
                Iter::new(&self.buf[start..], channels)
            }
        }
    }

    /// Construct a new [Channel] reference with a lifetime associated with the
    /// current channel instance instead of the underlying buffer.
    ///
    /// Most of the time it is not necessary to use this, since [Channel]
    /// implements [Copy] and its lifetime would coerce to any compatible
    /// lifetime. This method is currently just here for completeness sake.
    ///
    /// Both of these work equally well:
    ///
    /// ```rust
    /// use rotary::Channel;
    ///
    /// struct Foo<'a> {
    ///     channel: Channel<'a, i16>,
    /// }
    ///
    /// impl<'a> Foo<'a> {
    ///     fn channel(&self) -> Channel<'_, i16> {
    ///         self.channel.as_ref()
    ///     }
    ///
    ///     fn coerced_channel(&self) -> Channel<'_, i16> {
    ///         self.channel
    ///     }
    /// }
    /// ```
    #[inline]
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
    /// use rotary::{Channels as _, ChannelsMut as _};
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
            Kind::Linear => Self {
                buf: buf.get(n..).unwrap_or_default(),
                kind,
            },
            Kind::Interleaved { channels, .. } => Self {
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
    /// use rotary::{Channels as _, ChannelsMut as _};
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
            Kind::Linear => {
                let start = buf.len().saturating_sub(n);

                Self {
                    buf: buf.get(start..).unwrap_or_default(),
                    kind,
                }
            }
            Kind::Interleaved { channels, .. } => {
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
    /// use rotary::{Channels as _, ChannelsMut as _};
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
            Kind::Linear => Channel {
                buf: buf.get(..limit).unwrap_or_default(),
                kind,
            },
            Kind::Interleaved { channels, .. } => Channel {
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
            Kind::Linear => Channel {
                buf: buf.get(n..n + len).unwrap_or_default(),
                kind,
            },
            Kind::Interleaved { channels, .. } => {
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
    /// use rotary::Channels;
    ///
    /// fn test(buf: &dyn Channels<f32>) {
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
    /// use rotary::Channels;
    ///
    /// fn test(buf: &dyn Channels<f32>) {
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
            Kind::Linear => {
                let end = usize::min(out.len(), self.buf.len());
                out[..end].copy_from_slice(&self.buf[..end]);
            }
            Kind::Interleaved { channels, channel } => {
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
    /// use rotary::Channels;
    ///
    /// fn test(buf: &dyn Channels<f32>) {
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
            Kind::Linear => {
                for (o, f) in iter.into_iter().zip(self.buf) {
                    *o = *f;
                }
            }
            Kind::Interleaved { channels, channel } => {
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
            Kind::Linear => &self.buf[index],
            Kind::Interleaved { channels, channel } => &self.buf[channel + channels * index],
        }
    }
}

/// The mutable buffer of a single channel.
///
/// This doesn't provide direct access to the underlying buffer, but rather
/// allows us to copy data usinga  number of utility functions.
#[derive(Debug)]
pub struct ChannelMut<'a, T> {
    buf: &'a mut [T],
    kind: Kind,
}

impl<'a, T> ChannelMut<'a, T> {
    /// Construct a mutable linear channel buffer.
    ///
    /// The buffer provided as-is constitutes the frames of the channel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::ChannelMut;
    ///
    /// let buf = &mut [1, 3, 5, 7];
    /// let mut channel = ChannelMut::linear(buf);
    ///
    /// assert_eq!(channel[1], 3);
    /// assert_eq!(channel[2], 5);
    ///
    /// channel[1] *= 4;
    ///
    /// assert_eq!(buf, &[1, 12, 5, 7]);
    /// ```
    pub fn linear(buf: &'a mut [T]) -> Self {
        Self {
            buf,
            kind: Kind::Linear,
        }
    }

    /// Construct a mutable interleaved channel buffer.
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
    /// use rotary::ChannelMut;
    ///
    /// let buf = &mut [1, 2, 3, 4, 5, 6, 7, 8];
    /// let mut channel = ChannelMut::interleaved(buf, 2, 1);
    ///
    /// assert_eq!(channel[1], 4);
    /// assert_eq!(channel[2], 6);
    ///
    /// channel[1] *= 4;
    ///
    /// assert_eq!(buf, &[1, 2, 3, 16, 5, 6, 7, 8]);
    /// ```
    pub fn interleaved(buf: &'a mut [T], channels: usize, channel: usize) -> Self {
        Self {
            buf,
            kind: Kind::Interleaved { channels, channel },
        }
    }

    /// Convert the current mutable channel into a [Channel] with the lifetime
    /// matching the underlying buffer.
    ///
    /// This is required in order to fully convert a [ChannelMut] into a
    /// [Channel] with the lifetime associated with the buffer, because if we
    /// only use [as_ref][ChannelMut::as_ref] we'll actually be creating a
    /// reference to the mutable buffer instead.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::{Channel, ChannelMut};
    ///
    /// struct Foo<'a> {
    ///     channel: ChannelMut<'a, i16>,
    /// }
    ///
    /// impl<'a> Foo<'a> {
    ///     fn into_channel(self) -> Channel<'a, i16> {
    ///         self.channel.into_ref()
    ///     }
    /// }
    /// ```
    ///
    /// In contrast, this doesn't compile:
    ///
    /// ```rust,compile_fail
    /// use rotary::{Channel, ChannelMut};
    ///
    /// struct Foo<'a> {
    ///     channel: ChannelMut<'a, i16>,
    /// }
    ///
    /// impl<'a> Foo<'a> {
    ///     fn into_channel(self) -> Channel<'a, i16> {
    ///         self.channel.as_ref()
    ///     }
    /// }
    /// ```
    ///
    /// With the following error:
    ///
    /// ```text
    ///    error[E0515]: cannot return value referencing local data `self.channel`
    ///    --> test.rs:11:9
    ///     |
    ///  11 |         self.channel.as_ref()
    ///     |         ------------^^^^^^^^^
    ///     |         |
    ///     |         returns a value referencing data owned by the current function
    ///     |         `self.channel` is borrowed here
    ///```
    #[inline]
    pub fn into_ref(self) -> Channel<'a, T> {
        Channel {
            buf: self.buf,
            kind: self.kind,
        }
    }

    /// Construct a new [Channel] reference with a lifetime associated with the
    /// current channel instance instead of the underlying buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::{Channel, ChannelMut};
    ///
    /// let buf = &mut [1, 2, 3, 4];
    /// let channel = ChannelMut::linear(buf);
    ///
    /// let channel1 = channel.as_ref();
    /// let channel2 = channel1; // Channel is Copy.
    ///
    /// assert_eq!(channel1[0], channel2[0]);
    /// ```
    #[inline]
    pub fn as_ref(&self) -> Channel<'_, T> {
        Channel {
            buf: self.buf,
            kind: self.kind,
        }
    }

    /// Construct a new mutable channel reference with a lifetime associated
    /// with the current channel instance instead of the underlying buffer.
    ///
    /// Reborrowing like this is sometimes necessary, like if you want to pass
    /// an instance of [ChannelMut] directly into another function instead of
    /// borrowing it:
    ///
    /// ```rust
    /// use rotary::{ChannelsMut, ChannelMut};
    ///
    /// fn takes_channel_mut(mut channel: ChannelMut<'_, i16>) {
    ///     channel[1] = 42;
    /// }
    ///
    /// let mut buffer = rotary::interleaved![[0; 4]; 2];
    /// let mut channel = buffer.channel_mut(1);
    ///
    /// takes_channel_mut(channel.as_mut());
    ///
    /// assert_eq!(channel[1], 42);
    /// ```
    ///
    /// Without the reborrow, we would end up moving the channel:
    ///
    /// ```rust,compile_fail
    /// use rotary::{ChannelsMut, ChannelMut};
    ///
    /// fn takes_channel_mut(mut channel: ChannelMut<'_, i16>) {
    ///     channel[1] = 42;
    /// }
    ///
    /// let mut buffer = rotary::interleaved![[0; 4]; 2];
    /// let mut channel = buffer.channel_mut(1);
    ///
    /// takes_channel_mut(channel);
    ///
    /// assert_eq!(channel[1], 42);
    /// ```
    ///
    /// Causing the following error:
    ///
    /// ```text
    ///    error[E0382]: borrow of moved value: `channel`
    ///    --> test.rs:10:12
    ///     |
    ///  10 | let mut channel = buffer.channel_mut(1);
    ///     |     ----------- move occurs because `channel` has type `ChannelMut<'_, i16>`,
    ///     |                 which does not implement the `Copy` trait
    ///  11 |
    ///  12 | takes_channel_mut(channel);
    ///     |                   ------- value moved here
    ///  13 |
    ///  14 | assert_eq!(channel[1], 42);
    ///     |            ^^^^^^^ value borrowed here after move
    /// ```
    #[inline]
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
    /// use rotary::ChannelsMut;
    ///
    /// fn test(buf: &dyn ChannelsMut<f32>) {
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
            Kind::Linear => self.buf.len(),
            Kind::Interleaved { channels, .. } => self.buf.len() / channels,
        }
    }

    /// The number of chunks that can fit with the given size.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::ChannelsMut;
    ///
    /// fn test(buf: &dyn ChannelsMut<f32>) {
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
    /// use rotary::{Channels as _, ChannelsMut as _};
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
        self.into_ref().iter()
    }

    /// Construct a mutable iterator over the channel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::{Channels as _, ChannelsMut as _};
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
            Kind::Linear => IterMut::new(self.buf, 1),
            Kind::Interleaved { channels, channel } => {
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
    /// use rotary::{Channels as _, ChannelsMut as _};
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
            Kind::Linear => Self {
                buf: buf.get_mut(n..).unwrap_or_default(),
                kind,
            },
            Kind::Interleaved { channels, .. } => Self {
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
    /// use rotary::{Channels as _, ChannelsMut as _};
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
            Kind::Linear => {
                let start = buf.len().saturating_sub(n);

                Self {
                    buf: buf.get_mut(start..).unwrap_or_default(),
                    kind,
                }
            }
            Kind::Interleaved { channels, .. } => {
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
    /// use rotary::{Channels as _, ChannelsMut as _};
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
            Kind::Linear => Self {
                buf: buf.get_mut(..limit).unwrap_or_default(),
                kind,
            },
            Kind::Interleaved { channels, .. } => Self {
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
    /// use rotary::{Channels as _, ChannelsMut as _};
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
    /// use rotary::Channels;
    ///
    /// fn test(buf: &dyn Channels<f32>) {
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
            Kind::Linear => Self {
                buf: buf.get_mut(n..n + len).unwrap_or_default(),
                kind,
            },
            Kind::Interleaved { channels, .. } => {
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
    /// use rotary::ChannelsMut;
    ///
    /// fn test(buf: &mut dyn ChannelsMut<f32>) {
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
            Kind::Linear => {
                let len = usize::min(self.buf.len(), buf.len());
                self.buf[..len].copy_from_slice(&buf[..len]);
            }
            Kind::Interleaved { channels, channel } => {
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
    /// use rotary::ChannelsMut;
    ///
    /// fn test(buf: &mut dyn ChannelsMut<f32>) {
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
    /// use rotary::ChannelsMut;
    ///
    /// fn test(buf: &mut dyn ChannelsMut<f32>) {
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
            Kind::Linear => {
                for (o, f) in self.buf.iter_mut().zip(iter) {
                    *o = f;
                }
            }
            Kind::Interleaved { channels, channel } => {
                let buf = self.buf[channel..].iter_mut().step_by(channels);

                for (o, f) in buf.zip(iter) {
                    *o = f;
                }
            }
        }
    }

    /// Copy this channel from another.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::{Channels as _, ChannelsMut as _};
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
            (Kind::Linear, Kind::Linear) => {
                self.buf.copy_from_slice(&from.buf[..]);
            }
            _ => {
                for (o, f) in self.as_mut().iter_mut().zip(from) {
                    *o = f;
                }
            }
        }
    }

    /// Translate this channel from another.
    ///
    /// This will translate each sample in the channel through the appropriate
    /// [Translate] implementation.
    ///
    /// This is used for converting a buffer containing one type of sample into
    /// another.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::{Channels as _, ChannelsMut as _};
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
            Kind::Linear => &self.buf[index],
            Kind::Interleaved { channels, channel } => &self.buf[channel + channels * index],
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
/// use rotary::ChannelsMut;
///
/// fn test(buf: &mut dyn ChannelsMut<f32>) {
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
            Kind::Linear => &mut self.buf[index],
            Kind::Interleaved { channels, channel } => &mut self.buf[channel + channels * index],
        }
    }
}
