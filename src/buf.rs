use crate::range::Range;
use crate::sample::Sample;

/// A trait describing an immutable audio buffer.
pub trait Buf<T> {
    /// The number of channels in the buffer.
    fn channels(&self) -> usize;

    /// Test if the given channel is masked.
    fn is_masked(&self, channel: usize) -> bool;

    /// Return a handler to the buffer associated with the channel.
    ///
    /// Note that we don't access the buffer for the underlying channel directly
    /// as a linear buffer like `&[T]`, because the underlying representation
    /// might be different.
    ///
    /// We must instead make use of the various utility functions found on
    /// [BufChannel] to copy data out of the channel.
    ///
    /// # Panics
    ///
    /// Panics if the specified channel is out of bound as reported by
    /// [Buf::channels].
    fn channel(&self, channel: usize) -> BufChannel<'_, T>;
}

/// A trait describing a mutable audio buffer.
pub trait BufMut<T>: Buf<T> {
    /// Return a mutable handler to the buffer associated with the channel.
    ///
    /// # Panics
    ///
    /// Panics if the specified channel is out of bound as reported by
    /// [Buf::channels].
    fn channel_mut(&mut self, channel: usize) -> BufChannelMut<'_, T>;

    /// Resize the number of frames.
    fn resize(&mut self, frames: usize);

    /// Resize the buffer to match the given topology.
    fn resize_topology(&mut self, channels: usize, frames: usize);

    /// Set if the given channel is masked or not.
    fn set_masked(&mut self, channel: usize, masked: bool);
}

/// The buffer of a single channel.
///
/// This doesn't provide direct access to the underlying buffer, but rather
/// allows us to copy data usinga  number of utility functions.
#[derive(Debug, Clone, Copy)]
pub struct BufChannel<'a, T> {
    buf: &'a [T],
    kind: BufChannelKind,
}

impl<'a, T> BufChannel<'a, T> {
    /// Construct a linear buffer.
    pub fn linear(buf: &'a [T]) -> Self {
        Self {
            buf,
            kind: BufChannelKind::Linear,
        }
    }

    /// Construct an interleaved buffer.
    pub fn interleaved(buf: &'a [T], channels: usize, channel: usize) -> Self {
        Self {
            buf,
            kind: BufChannelKind::Interleaved { channels, channel },
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
            BufChannelKind::Linear => self.buf.len(),
            BufChannelKind::Interleaved { channels, .. } => self.buf.len() / channels,
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
            BufChannelKind::Linear => {
                out.copy_from_slice(self.buf);
            }
            BufChannelKind::Interleaved { channels, channel } => {
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
            BufChannelKind::Linear => {
                let buf = &self.buf[len * n..];
                let end = usize::min(buf.len(), len);
                let end = usize::min(end, out.len());
                out[..end].copy_from_slice(&buf[..end]);
            }
            BufChannelKind::Interleaved { channels, channel } => {
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
            BufChannelKind::Linear => {
                for (o, f) in iter.into_iter().zip(self.buf) {
                    *o = *f;
                }
            }
            BufChannelKind::Interleaved { channels, channel } => {
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
            BufChannelKind::Linear => {
                for (f, s) in self.buf.iter().enumerate() {
                    out[m(f)] = *s;
                }
            }
            BufChannelKind::Interleaved { channels, channel } => {
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
            BufChannelKind::Linear => {
                let buf = &self.buf[start..];
                out.copy_from_slice(&buf[..len]);
            }
            BufChannelKind::Interleaved { channels, channel } => {
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

/// The mutable buffer of a single channel.
///
/// This doesn't provide direct access to the underlying buffer, but rather
/// allows us to copy data usinga  number of utility functions.
#[derive(Debug)]
pub struct BufChannelMut<'a, T> {
    buf: &'a mut [T],
    kind: BufChannelKind,
}

impl<'a, T> BufChannelMut<'a, T> {
    /// Construct a linear buffer.
    pub fn linear(buf: &'a mut [T]) -> Self {
        Self {
            buf,
            kind: BufChannelKind::Linear,
        }
    }

    /// Construct an interleaved buffer.
    pub fn interleaved(buf: &'a mut [T], channels: usize, channel: usize) -> Self {
        Self {
            buf,
            kind: BufChannelKind::Interleaved { channels, channel },
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
            BufChannelKind::Linear => self.buf.len(),
            BufChannelKind::Interleaved { channels, .. } => self.buf.len() / channels,
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
            BufChannelKind::Linear => {
                self.buf[n] = value;
            }
            BufChannelKind::Interleaved { channels, channel } => {
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
            BufChannelKind::Linear => {
                let len = usize::min(self.buf.len(), buf.len());
                self.buf[..len].copy_from_slice(&buf[..len]);
            }
            BufChannelKind::Interleaved { channels, channel } => {
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
            BufChannelKind::Linear => {
                for (o, f) in range.map_mut_linear(self.buf).iter_mut().zip(iter) {
                    *o = f;
                }
            }
            BufChannelKind::Interleaved { channels, channel } => {
                let buf = self.buf[channel..].iter_mut().step_by(channels);
                range.map_iter_interleaved(buf, iter, |(o, f)| *o = f);
            }
        }
    }
}

impl<T> Buf<T> for Vec<Vec<T>> {
    fn channels(&self) -> usize {
        self.len()
    }

    fn is_masked(&self, channel: usize) -> bool {
        self[channel].is_empty()
    }

    fn channel(&self, channel: usize) -> BufChannel<'_, T> {
        BufChannel::linear(&self[channel])
    }
}

impl<T> BufMut<T> for Vec<Vec<T>>
where
    T: Sample,
{
    fn channel_mut(&mut self, channel: usize) -> BufChannelMut<'_, T> {
        BufChannelMut::linear(&mut self[channel])
    }

    fn resize(&mut self, frames: usize) {
        for buf in self.iter_mut() {
            buf.resize(frames, T::ZERO);
        }
    }

    fn resize_topology(&mut self, channels: usize, frames: usize) {
        for buf in self.iter_mut() {
            buf.resize(frames, T::ZERO);
        }

        for _ in self.len()..channels {
            self.push(vec![T::ZERO; frames]);
        }
    }

    fn set_masked(&mut self, _: usize, _: bool) {}
}

impl<T> Buf<T> for [Vec<T>] {
    fn channels(&self) -> usize {
        self.as_ref().len()
    }

    fn is_masked(&self, channel: usize) -> bool {
        self.as_ref()[channel].is_empty()
    }

    fn channel(&self, channel: usize) -> BufChannel<'_, T> {
        BufChannel::linear(&self.as_ref()[channel])
    }
}

/// Used to determine how a buffer is indexed.
#[derive(Debug, Clone, Copy)]
enum BufChannelKind {
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
