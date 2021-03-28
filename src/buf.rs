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
    /// test(&rotary::audio_buffer![[0.0; 16]; 2]);
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
    /// test(&rotary::audio_buffer![[0.0; 16]; 2]);
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
    /// test(&rotary::audio_buffer![[1.0; 16]; 2]);
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
    /// test(&rotary::audio_buffer![[1.0; 16]; 2]);
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
    /// test(&rotary::audio_buffer![[1.0; 16]; 2]);
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
    /// test(&rotary::audio_buffer![[1.0; 16]; 2]);
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
}

/// The simple vector of vectors buffer where an empty vector indicates that the
/// channel is masked.
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

/// The simple slice of vectors buffer where an empty vector indicates that the
/// channel is masked.
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
