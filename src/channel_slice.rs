//! A channel buffer as created through [Buf::channel][crate::Buf::channel].

use crate::buf::ChannelSliceKind;
use crate::range::Range;
use crate::sample::Translate;

mod iter;
pub use self::iter::{Iter, IterMut};
use crate::sample::Sample;

/// The buffer of a single channel.
///
/// This doesn't provide direct access to the underlying buffer, but rather
/// allows us to copy data usinga  number of utility functions.
#[derive(Debug, Clone, Copy)]
pub struct ChannelSlice<'a, T>
where
    T: Sample,
{
    pub(crate) buf: &'a [T],
    pub(crate) kind: ChannelSliceKind,
}

impl<'a, T> ChannelSlice<'a, T>
where
    T: Sample,
{
    /// Construct a linear buffer.
    pub fn linear(buf: &'a [T]) -> Self {
        Self {
            buf,
            kind: ChannelSliceKind::Linear,
        }
    }

    /// Construct an interleaved buffer.
    pub fn interleaved(buf: &'a [T], channels: usize, channel: usize) -> Self {
        Self {
            buf,
            kind: ChannelSliceKind::Interleaved { channels, channel },
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
            ChannelSliceKind::Linear => self.buf.len(),
            ChannelSliceKind::Interleaved { channels, .. } => self.buf.len() / channels,
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
    pub fn iter(&self) -> Iter<'_, T> {
        match self.kind {
            ChannelSliceKind::Linear => Iter::new(self.buf, 1),
            ChannelSliceKind::Interleaved { channels, channel } => {
                let start = usize::min(channel, self.buf.len());
                Iter::new(&self.buf[start..], channels)
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
            ChannelSliceKind::Linear => {
                out.copy_from_slice(self.buf);
            }
            ChannelSliceKind::Interleaved { channels, channel } => {
                for (o, f) in out
                    .iter_mut()
                    .zip(self.buf[channel..].iter().step_by(channels))
                {
                    *o = *f;
                }
            }
        }
    }

    /// Copy the given chunk of a channel into a buffer.
    ///
    /// The length of the chunk to copy is determined by `len`. The offset of
    /// the chunk to copy is determined by `n`, where `n` is the number of `len`
    /// sized chunks.
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
    ///     channel.copy_chunk(&mut buf[..], 3, 4);
    ///
    ///     assert!(buf.iter().all(|f| *f == 1.0));
    /// }
    ///
    /// test(&rotary::dynamic![[1.0; 16]; 2]);
    /// test(&rotary::sequential![[1.0; 16]; 2]);
    /// test(&rotary::interleaved![[1.0; 16]; 2]);
    /// ```
    pub fn copy_chunk(&self, out: &mut [T], n: usize, len: usize)
    where
        T: Copy,
    {
        match self.kind {
            ChannelSliceKind::Linear => {
                let buf = &self.buf[len * n..];
                let end = usize::min(buf.len(), len);
                let end = usize::min(end, out.len());
                out[..end].copy_from_slice(&buf[..end]);
            }
            ChannelSliceKind::Interleaved { channels, channel } => {
                let start = len * n;
                let it = self.buf[channel + start..]
                    .iter()
                    .step_by(channels)
                    .take(len);

                for (o, f) in out.iter_mut().zip(it) {
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
            ChannelSliceKind::Linear => {
                for (o, f) in iter.into_iter().zip(self.buf) {
                    *o = *f;
                }
            }
            ChannelSliceKind::Interleaved { channels, channel } => {
                for (o, f) in iter
                    .into_iter()
                    .zip(self.buf[channel..].iter().step_by(channels))
                {
                    *o = *f;
                }
            }
        }
    }

    /// Copy into the given slice, mapping the index by the given mapping
    /// function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::Buf;
    ///
    /// fn test(buf: &dyn Buf<f32>) {
    ///     let channel = buf.channel(0);
    ///
    ///     let mut buf = vec![0.0; channel.frames() * 2];
    ///
    ///     // Copy into every other position in `buf`.
    ///     channel.map_into_slice(&mut buf[..], |n| n * 2);
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
    pub fn map_into_slice<M>(&self, out: &mut [T], m: M)
    where
        M: Fn(usize) -> usize,
        T: Copy,
    {
        match self.kind {
            ChannelSliceKind::Linear => {
                for (f, s) in self.buf.iter().enumerate() {
                    out[m(f)] = *s;
                }
            }
            ChannelSliceKind::Interleaved { channels, channel } => {
                for (f, s) in self.buf[channel..].iter().step_by(channels).enumerate() {
                    out[m(f)] = *s;
                }
            }
        }
    }

    /// Copy the given range into a slice.
    ///
    /// The range to be copied is designated with a starting position `start`
    /// and a length `len`.
    pub fn range_into_slice(&self, out: &mut [T], start: usize, len: usize)
    where
        T: Copy,
    {
        match self.kind {
            ChannelSliceKind::Linear => {
                let buf = &self.buf[start..];
                out.copy_from_slice(&buf[..len]);
            }
            ChannelSliceKind::Interleaved { channels, channel } => {
                for (o, f) in out
                    .iter_mut()
                    .zip(self.buf[start * channels + channel..].iter().take(len))
                {
                    *o = *f;
                }
            }
        }
    }
}

impl<'a, T> IntoIterator for ChannelSlice<'a, T>
where
    T: Sample,
{
    type Item = T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        match self.kind {
            ChannelSliceKind::Linear => Iter::new(self.buf, 1),
            ChannelSliceKind::Interleaved { channels, channel } => {
                let start = usize::min(channel, self.buf.len());
                Iter::new(&self.buf[start..], channels)
            }
        }
    }
}

impl<'a, T> IntoIterator for &'a ChannelSlice<'_, T>
where
    T: Sample,
{
    type Item = T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// The mutable buffer of a single channel.
///
/// This doesn't provide direct access to the underlying buffer, but rather
/// allows us to copy data usinga  number of utility functions.
#[derive(Debug)]
pub struct ChannelSliceMut<'a, T>
where
    T: Sample,
{
    pub(crate) buf: &'a mut [T],
    pub(crate) kind: ChannelSliceKind,
}

impl<'a, T> ChannelSliceMut<'a, T>
where
    T: Sample,
{
    /// Construct a linear buffer.
    pub fn linear(buf: &'a mut [T]) -> Self {
        Self {
            buf,
            kind: ChannelSliceKind::Linear,
        }
    }

    /// Construct an interleaved buffer.
    pub fn interleaved(buf: &'a mut [T], channels: usize, channel: usize) -> Self {
        Self {
            buf,
            kind: ChannelSliceKind::Interleaved { channels, channel },
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
    pub fn iter(&self) -> Iter<'_, T> {
        match self.kind {
            ChannelSliceKind::Linear => Iter::new(self.buf, 1),
            ChannelSliceKind::Interleaved { channels, channel } => {
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
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        match self.kind {
            ChannelSliceKind::Linear => IterMut::new(self.buf, 1),
            ChannelSliceKind::Interleaved { channels, channel } => {
                let start = usize::min(channel, self.buf.len());
                IterMut::new(&mut self.buf[start..], channels)
            }
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
            ChannelSliceKind::Linear => self.buf.len(),
            ChannelSliceKind::Interleaved { channels, .. } => self.buf.len() / channels,
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

    /// Set the value at the given frame in the current channel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::BufMut;
    /// use rotary::Range as _;
    ///
    /// fn test(buf: &mut dyn BufMut<f32>) {
    ///     buf.channel_mut(0).set(1, 1.0);
    ///     buf.channel_mut(0).set(7, 1.0);
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
    pub fn set(&mut self, n: usize, value: T) {
        match self.kind {
            ChannelSliceKind::Linear => {
                self.buf[n] = value;
            }
            ChannelSliceKind::Interleaved { channels, channel } => {
                self.buf[channel + channels * n] = value;
            }
        }
    }

    /// Copy from the given slice.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::BufMut;
    /// use rotary::Range as _;
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
            ChannelSliceKind::Linear => {
                let len = usize::min(self.buf.len(), buf.len());
                self.buf[..len].copy_from_slice(&buf[..len]);
            }
            ChannelSliceKind::Interleaved { channels, channel } => {
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
    /// use rotary::Range as _;
    ///
    /// fn test(buf: &mut dyn BufMut<f32>) {
    ///     buf.channel_mut(0).copy_from_iter(rotary::range::full().offset(2), vec![1.0; 4]);
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
    /// use rotary::Range as _;
    ///
    /// fn test(buf: &mut dyn BufMut<f32>) {
    ///     buf.channel_mut(0).copy_from_iter(rotary::range::full().offset(2).chunk(0, 2), vec![1.0; 4]);
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
    pub fn copy_from_iter<R, I>(&mut self, range: R, iter: I)
    where
        R: Range,
        I: IntoIterator<Item = T>,
    {
        match self.kind {
            ChannelSliceKind::Linear => {
                for (o, f) in range.map_mut_linear(self.buf).iter_mut().zip(iter) {
                    *o = f;
                }
            }
            ChannelSliceKind::Interleaved { channels, channel } => {
                let buf = self.buf[channel..].iter_mut().step_by(channels);
                range.map_iter_interleaved(buf, iter, |(o, f)| *o = f);
            }
        }
    }

    /// Copy from another channel.
    pub fn copy_from(&mut self, from: ChannelSlice<T>) {
        match (self.kind, from.kind) {
            (ChannelSliceKind::Linear, ChannelSliceKind::Linear) => {
                self.buf.copy_from_slice(&from.buf[..]);
            }
            (ChannelSliceKind::Linear, ChannelSliceKind::Interleaved { channels, channel }) => {
                let from = from.buf[channel..].iter().step_by(channels);

                for (o, f) in self.buf.iter_mut().zip(from) {
                    *o = *f;
                }
            }
            (ChannelSliceKind::Interleaved { channels, channel }, ChannelSliceKind::Linear) => {
                let to = self.buf[channel..].iter_mut().step_by(channels);

                for (o, f) in to.zip(from.buf) {
                    *o = *f;
                }
            }
            (
                ChannelSliceKind::Interleaved { channels, channel },
                ChannelSliceKind::Interleaved {
                    channels: from_channels,
                    channel: from_channel,
                },
            ) => {
                let to = self.buf[channel..].iter_mut().step_by(channels);
                let from = from.buf[from_channel..].iter().step_by(from_channels);

                for (o, f) in to.zip(from) {
                    *o = *f;
                }
            }
        }
    }

    /// Translate from another channel.
    pub fn translate_from<U>(&mut self, from: ChannelSlice<U>)
    where
        U: Sample,
        T: Translate<U>,
    {
        match (self.kind, from.kind) {
            (ChannelSliceKind::Linear, ChannelSliceKind::Linear) => {
                for (o, f) in self.buf.iter_mut().zip(from.buf) {
                    *o = T::translate(*f);
                }
            }
            (ChannelSliceKind::Linear, ChannelSliceKind::Interleaved { channels, channel }) => {
                let from = from.buf[channel..].iter().step_by(channels);

                for (o, f) in self.buf.iter_mut().zip(from) {
                    *o = T::translate(*f);
                }
            }
            (ChannelSliceKind::Interleaved { channels, channel }, ChannelSliceKind::Linear) => {
                let to = self.buf[channel..].iter_mut().step_by(channels);

                for (o, f) in to.zip(from.buf) {
                    *o = T::translate(*f);
                }
            }
            (
                ChannelSliceKind::Interleaved { channels, channel },
                ChannelSliceKind::Interleaved {
                    channels: from_channels,
                    channel: from_channel,
                },
            ) => {
                let to = self.buf[channel..].iter_mut().step_by(channels);
                let from = from.buf[from_channel..].iter().step_by(from_channels);

                for (o, f) in to.zip(from) {
                    *o = T::translate(*f);
                }
            }
        }
    }
}

impl<'a, T> IntoIterator for ChannelSliceMut<'a, T>
where
    T: Sample,
{
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        match self.kind {
            ChannelSliceKind::Linear => IterMut::new(self.buf, 1),
            ChannelSliceKind::Interleaved { channels, channel } => {
                let start = usize::min(channel, self.buf.len());
                IterMut::new(&mut self.buf[start..], channels)
            }
        }
    }
}

impl<'a, T> IntoIterator for &'a mut ChannelSliceMut<'_, T>
where
    T: Sample,
{
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        (*self).iter_mut()
    }
}
