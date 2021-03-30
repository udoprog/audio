//! Trait for dealing with abstract channel buffers.

use crate::channel_slice::{ChannelSlice, ChannelSliceMut};
use crate::sample::Sample;

mod offset;
pub use self::offset::Offset;

mod limit;
pub use self::limit::Limit;

/// A trait describing an immutable audio buffer.
pub trait Buf<T>
where
    T: Sample,
{
    /// The number of frames in a buffer.
    fn frames(&self) -> usize;

    /// The number of channels in the buffer.
    fn channels(&self) -> usize;

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
    fn channel(&self, channel: usize) -> ChannelSlice<'_, T>;

    /// Offset the buffer to process by `offset` number of frames.
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
    /// let mut to = rotary::Interleaved::<f32>::with_topology(2, 4);
    ///
    /// to.channel_mut(0).copy_from((&from).offset(2).channel(0));
    /// assert_eq!(to.as_slice(), &[1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    /// ```
    ///
    /// Test with a mutable buffer.
    ///
    /// ```rust
    /// use rotary::{Buf as _, BufMut as _};
    ///
    /// let mut buffer = rotary::Interleaved::with_topology(2, 4);
    ///
    /// (&mut buffer).offset(2).channel_mut(0).copy_from_slice(&[1.0, 1.0]);
    ///
    /// assert_eq!(buffer.as_slice(), &[0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0])
    /// ```
    fn offset(self, offset: usize) -> Offset<Self>
    where
        Self: Sized,
    {
        Offset::new(self, offset)
    }

    /// Limit the buffer to process by `limit` number of frames.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::{Buf as _, BufMut as _};
    ///
    /// let from = rotary::interleaved![[1.0f32; 4]; 2];
    /// let mut to = rotary::Interleaved::<f32>::with_topology(2, 4);
    ///
    /// to.channel_mut(0).copy_from(from.limit(2).channel(0));
    /// assert_eq!(to.as_slice(), &[1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    /// ```
    fn limit(self, limit: usize) -> Limit<Self>
    where
        Self: Sized,
    {
        Limit::new(self, limit)
    }
}

impl<B, T> Buf<T> for &B
where
    B: Buf<T>,
    T: Sample,
{
    fn frames(&self) -> usize {
        (**self).frames()
    }

    fn channels(&self) -> usize {
        (**self).channels()
    }

    fn channel(&self, channel: usize) -> ChannelSlice<'_, T> {
        (**self).channel(channel)
    }
}

/// A trait describing a mutable audio buffer.
pub trait BufMut<T>: Buf<T>
where
    T: Sample,
{
    /// Return a mutable handler to the buffer associated with the channel.
    ///
    /// # Panics
    ///
    /// Panics if the specified channel is out of bound as reported by
    /// [Buf::channels].
    fn channel_mut(&mut self, channel: usize) -> ChannelSliceMut<'_, T>;

    /// Resize the number of frames in the buffer.
    fn resize(&mut self, frames: usize);

    /// Resize the buffer to match the given topology.
    fn resize_topology(&mut self, channels: usize, frames: usize);
}

impl<B, T> Buf<T> for &mut B
where
    B: ?Sized + Buf<T>,
    T: Sample,
{
    fn frames(&self) -> usize {
        (**self).frames()
    }

    fn channels(&self) -> usize {
        (**self).channels()
    }

    fn channel(&self, channel: usize) -> ChannelSlice<'_, T> {
        (**self).channel(channel)
    }
}

impl<B, T> BufMut<T> for &mut B
where
    B: ?Sized + BufMut<T>,
    T: Sample,
{
    fn channel_mut(&mut self, channel: usize) -> ChannelSliceMut<'_, T> {
        (**self).channel_mut(channel)
    }

    fn resize(&mut self, frames: usize) {
        (**self).resize(frames);
    }

    fn resize_topology(&mut self, channels: usize, frames: usize) {
        (**self).resize_topology(channels, frames);
    }
}

impl<T> Buf<T> for Vec<Vec<T>>
where
    T: Sample,
{
    fn frames(&self) -> usize {
        self.iter().map(|vec| vec.len()).next().unwrap_or_default()
    }

    fn channels(&self) -> usize {
        self.len()
    }

    fn channel(&self, channel: usize) -> ChannelSlice<'_, T> {
        ChannelSlice::linear(&self[channel])
    }
}

impl<T> BufMut<T> for Vec<Vec<T>>
where
    T: Sample,
{
    fn channel_mut(&mut self, channel: usize) -> ChannelSliceMut<'_, T> {
        ChannelSliceMut::linear(&mut self[channel])
    }

    fn resize(&mut self, frames: usize) {
        for buf in self.iter_mut() {
            buf.resize(frames, T::ZERO);
        }
    }

    fn resize_topology(&mut self, channels: usize, frames: usize) {
        for buf in self.iter_mut() {
            // channel is masked, so ignore.
            if buf.is_empty() {
                continue;
            }

            buf.resize(frames, T::ZERO);
        }

        for _ in self.len()..channels {
            self.push(vec![T::ZERO; frames]);
        }
    }
}

impl<T> Buf<T> for [Vec<T>]
where
    T: Sample,
{
    fn frames(&self) -> usize {
        self.as_ref()
            .iter()
            .map(|c| c.len())
            .next()
            .unwrap_or_default()
    }

    fn channels(&self) -> usize {
        self.as_ref().len()
    }

    fn channel(&self, channel: usize) -> ChannelSlice<'_, T> {
        ChannelSlice::linear(&self.as_ref()[channel])
    }
}

/// Used to determine how a buffer is indexed.
#[derive(Debug, Clone, Copy)]
pub(crate) enum ChannelSliceKind {
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
