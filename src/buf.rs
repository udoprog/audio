use crate::sample::Sample;

/// A trait describing an immutable audio buffer.
pub trait Buf<T> {
    /// How a channel is indexed.
    fn index(&self) -> BufIndex;

    /// The number of channels in the buffer.
    fn channels(&self) -> usize;

    /// Test if the given channel is masked.
    fn is_masked(&self, channel: usize) -> bool;

    /// Return the buffer associated with the given channel.
    ///
    /// Note that the buffer might not be linear, and should be processed
    /// according to the mode provided in [index].
    ///
    /// # Panics
    ///
    /// Panics if the specified channel is out of bound.
    fn channel(&self, channel: usize) -> &[T];
}

/// The default vector of vectors buffer.
impl<T> Buf<T> for Vec<Vec<T>> {
    fn index(&self) -> BufIndex {
        BufIndex::Linear
    }

    fn channels(&self) -> usize {
        self.len()
    }

    fn is_masked(&self, channel: usize) -> bool {
        self[channel].is_empty()
    }

    fn channel(&self, channel: usize) -> &[T] {
        &self[channel]
    }
}

/// Used to determine how a buffer is indexed.
#[derive(Debug, Clone, Copy)]
pub enum BufIndex {
    /// Returned channel buffer is indexed in a linear manner.
    Linear,
    /// Returned channel buffer is indexed in an interleaved manner.
    Interleaved {
        /// The number of channels in the interleaved buffer.
        channels: usize,
    },
}

impl BufIndex {
    /// Access the length of the provided buffer.
    pub fn len<T>(&self, buf: &[T]) -> usize
    where
        T: Sample,
    {
        match self {
            BufIndex::Linear => buf.len(),
            BufIndex::Interleaved { channels } => buf.len() / channels,
        }
    }

    /// How many chunks of the given size can you divide buf into.
    ///
    /// This includes one extra chunk even if the chunk doesn't divide the frame
    /// length evenly.
    pub fn chunks<T>(&self, buf: &[T], chunk: usize) -> usize
    where
        T: Sample,
    {
        let len = self.len(buf);

        if len % chunk == 0 {
            len / chunk
        } else {
            len / chunk + 1
        }
    }

    /// Copy into the given slice of output.
    pub fn copy_into_slice<T>(&self, buf: &[T], channel: usize, out: &mut [T])
    where
        T: Sample,
    {
        match self {
            BufIndex::Linear => {
                out.copy_from_slice(buf);
            }
            BufIndex::Interleaved { channels } => {
                for (o, f) in out.iter_mut().zip(buf[channel..].iter().step_by(*channels)) {
                    *o = *f;
                }
            }
        }
    }

    /// Copy into the given slice of output.
    pub fn copy_chunk<T>(
        &self,
        buf: &[T],
        channel: usize,
        index: usize,
        chunk: usize,
        out: &mut [T],
    ) where
        T: Sample,
    {
        match self {
            BufIndex::Linear => {
                let buf = &buf[chunk * index..];
                let end = usize::min(buf.len(), chunk);
                let end = usize::min(end, out.len());
                out[..end].copy_from_slice(&buf[..end]);
            }
            BufIndex::Interleaved { channels } => {
                let start = chunk * index;
                let it = buf[channel + start..].iter().step_by(*channels).take(chunk);

                for (o, f) in out.iter_mut().zip(it) {
                    *o = *f;
                }
            }
        }
    }

    /// Copy into the given integer.
    pub fn copy_into_iter<'a, T, I>(&self, buf: &[T], channel: usize, iter: I)
    where
        T: 'a + Sample,
        I: IntoIterator<Item = &'a mut T>,
    {
        match self {
            BufIndex::Linear => {
                for (o, f) in iter.into_iter().zip(buf) {
                    *o = *f;
                }
            }
            BufIndex::Interleaved { channels } => {
                for (o, f) in iter
                    .into_iter()
                    .zip(buf[channel..].iter().step_by(*channels))
                {
                    *o = *f;
                }
            }
        }
    }

    /// Copy into the given integer.
    pub fn map_into_slice<'a, T, M>(&self, buf: &[T], channel: usize, out: &mut [T], m: M)
    where
        T: 'a + Sample,
        M: Fn(usize) -> usize,
    {
        match self {
            BufIndex::Linear => {
                for (f, s) in buf.iter().enumerate() {
                    out[m(f)] = *s;
                }
            }
            BufIndex::Interleaved { channels } => {
                for (f, s) in buf[channel..].iter().step_by(*channels).enumerate() {
                    out[m(f)] = *s;
                }
            }
        }
    }
}
