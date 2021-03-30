//! Trait for dealing with abstract channel buffers.

use crate::channel::{Channel, ChannelMut};
use crate::sample::Sample;

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
    /// [Channel] to copy data out of the channel.
    ///
    /// # Panics
    ///
    /// Panics if the specified channel is out of bound as reported by
    /// [Buf::channels].
    fn channel(&self, channel: usize) -> Channel<'_, T>;
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

    fn channel(&self, channel: usize) -> Channel<'_, T> {
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
    fn channel_mut(&mut self, channel: usize) -> ChannelMut<'_, T>;

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

    fn channel(&self, channel: usize) -> Channel<'_, T> {
        (**self).channel(channel)
    }
}

impl<B, T> BufMut<T> for &mut B
where
    B: ?Sized + BufMut<T>,
    T: Sample,
{
    fn channel_mut(&mut self, channel: usize) -> ChannelMut<'_, T> {
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

    fn channel(&self, channel: usize) -> Channel<'_, T> {
        Channel::linear(&self[channel])
    }
}

impl<T> BufMut<T> for Vec<Vec<T>>
where
    T: Sample,
{
    fn channel_mut(&mut self, channel: usize) -> ChannelMut<'_, T> {
        ChannelMut::linear(&mut self[channel])
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
