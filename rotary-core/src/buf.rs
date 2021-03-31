//! Trait for dealing with abstract channel buffers.

use crate::channel::{Channel, ChannelMut};
use crate::sample::Sample;

mod skip;
pub use self::skip::Skip;

mod limit;
pub use self::limit::Limit;

mod chunk;
pub use self::chunk::Chunk;

mod tail;
pub use self::tail::Tail;

/// Information on the current buffer.
pub trait BufInfo {
    /// The number of frames in a buffer.
    fn buf_info_frames(&self) -> usize;

    /// The number of channels in the buffer.
    fn buf_info_channels(&self) -> usize;
}

/// Trait implemented for buffers that can be resized.
pub trait ResizableBuf {
    /// Resize the number of frames in the buffer.
    fn resize(&mut self, frames: usize);

    /// Resize the buffer to match the given topology.
    fn resize_topology(&mut self, channels: usize, frames: usize);
}

/// A trait describing an immutable audio buffer.
pub trait Buf<T>: BufInfo {
    /// The number of frames in a buffer.
    fn frames(&self) -> usize {
        BufInfo::buf_info_frames(self)
    }

    /// The number of channels in the buffer.
    fn channels(&self) -> usize {
        BufInfo::buf_info_channels(self)
    }

    /// Return a handler to the buffer associated with the channel.
    ///
    /// Note that we don't access the buffer for the underlying channel directly
    /// as a linear buffer like `&[T]`, because the underlying representation
    /// might be different.
    ///
    /// We must instead make use of the various utility functions found on
    /// [Channel] to copy data out of the channel.
    ///
    /// # Panics
    ///
    /// Panics if the specified channel is out of bound as reported by
    /// [Buf::channels].
    fn channel(&self, channel: usize) -> Channel<'_, T>;

    /// Construct a new buffer where `n` frames are skipped.
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
    /// to.channel_mut(0).copy_from((&from).skip(2).channel(0));
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
    /// (&mut buffer).skip(2).channel_mut(0).copy_from_slice(&[1.0, 1.0]);
    ///
    /// assert_eq!(buffer.as_slice(), &[0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0])
    /// ```
    fn skip(self, n: usize) -> Skip<Self>
    where
        Self: Sized,
    {
        Skip::new(self, n)
    }

    /// Construct a new buffer where `n` frames are skipped.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::{Buf as _, BufMut as _};
    ///
    /// let from = rotary::interleaved![[1.0f32; 4]; 2];
    /// let mut to = rotary::interleaved![[0.0f32; 4]; 2];
    ///
    /// rotary::utils::copy(from, (&mut to).tail(2));
    ///
    /// assert_eq!(to.as_slice(), &[0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]);
    /// ```
    fn tail(self, n: usize) -> Tail<Self>
    where
        Self: Sized,
    {
        Tail::new(self, n)
    }

    /// Limit the channel buffer to `limit` number of frames.
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
    /// (&mut to).chunk(1, 2).channel_mut(0).copy_from(from.channel(0));
    /// assert_eq!(to.as_slice(), &[0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0]);
    /// ```
    fn chunk(self, n: usize, len: usize) -> Chunk<Self>
    where
        Self: Sized,
    {
        Chunk::new(self, n, len)
    }
}

impl<B> BufInfo for &B
where
    B: ?Sized + BufInfo,
{
    fn buf_info_frames(&self) -> usize {
        (**self).buf_info_frames()
    }

    fn buf_info_channels(&self) -> usize {
        (**self).buf_info_channels()
    }
}

impl<B, T> Buf<T> for &B
where
    B: Buf<T>,
{
    fn channel(&self, channel: usize) -> Channel<'_, T> {
        (**self).channel(channel)
    }
}

/// A trait describing a mutable audio buffer.
pub trait BufMut<T>: Buf<T> {
    /// Return a mutable handler to the buffer associated with the channel.
    ///
    /// # Panics
    ///
    /// Panics if the specified channel is out of bound as reported by
    /// [Buf::channels].
    fn channel_mut(&mut self, channel: usize) -> ChannelMut<'_, T>;
}

impl<B> BufInfo for &mut B
where
    B: ?Sized + BufInfo,
{
    fn buf_info_frames(&self) -> usize {
        (**self).buf_info_frames()
    }

    fn buf_info_channels(&self) -> usize {
        (**self).buf_info_channels()
    }
}

impl<B, T> Buf<T> for &mut B
where
    B: ?Sized + Buf<T>,
{
    fn channel(&self, channel: usize) -> Channel<'_, T> {
        (**self).channel(channel)
    }
}

impl<B> ResizableBuf for &mut B
where
    B: ?Sized + ResizableBuf,
{
    fn resize(&mut self, frames: usize) {
        (**self).resize(frames);
    }

    fn resize_topology(&mut self, channels: usize, frames: usize) {
        (**self).resize_topology(channels, frames);
    }
}

impl<B, T> BufMut<T> for &mut B
where
    B: ?Sized + BufMut<T>,
{
    fn channel_mut(&mut self, channel: usize) -> ChannelMut<'_, T> {
        (**self).channel_mut(channel)
    }
}

impl<T> BufInfo for Vec<Vec<T>> {
    fn buf_info_frames(&self) -> usize {
        self.iter().map(|vec| vec.len()).next().unwrap_or_default()
    }

    fn buf_info_channels(&self) -> usize {
        self.len()
    }
}

impl<T> Buf<T> for Vec<Vec<T>> {
    fn channel(&self, channel: usize) -> Channel<'_, T> {
        Channel::linear(&self[channel])
    }
}

impl<T> ResizableBuf for Vec<Vec<T>>
where
    T: Sample,
{
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
}

impl<T> BufMut<T> for Vec<Vec<T>> {
    fn channel_mut(&mut self, channel: usize) -> ChannelMut<'_, T> {
        ChannelMut::linear(&mut self[channel])
    }
}

impl<T> BufInfo for [Vec<T>] {
    fn buf_info_frames(&self) -> usize {
        self.as_ref().first().map(|c| c.len()).unwrap_or_default()
    }

    fn buf_info_channels(&self) -> usize {
        self.as_ref().len()
    }
}

impl<T> Buf<T> for [Vec<T>] {
    fn channel(&self, channel: usize) -> Channel<'_, T> {
        Channel::linear(&self.as_ref()[channel])
    }
}

/// Used to determine how a buffer is indexed.
#[derive(Debug, Clone, Copy)]
pub(crate) enum ChannelKind {
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
