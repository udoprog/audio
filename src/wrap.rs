//! Wrap an external type to implement [Buf] and [BufMut].

use crate::buf::{Buf, BufInfo, BufMut, ReadBuf, ResizableBuf};
use crate::channel::{Channel, ChannelMut};
use crate::sample::Sample;

/// Trait used for getting generic information on slices.
pub trait Slice {
    /// The length of the slice.
    fn slice_len(&self) -> usize;
}

impl<T> Slice for &'_ [T] {
    #[inline]
    fn slice_len(&self) -> usize {
        self.len()
    }
}

impl<T> Slice for &'_ mut [T] {
    #[inline]
    fn slice_len(&self) -> usize {
        self.len()
    }
}

/// Wrap a `value` as an interleaved buffer with the given number of channels.
pub fn interleaved<T>(value: T, channels: usize) -> Interleaved<T> {
    Interleaved { value, channels }
}

/// Wrap a `value` as a sequential buffer with the given number of frames. The
/// length of the buffer determines the number of channels.
pub fn sequential<T>(value: T, frames: usize) -> Sequential<T> {
    Sequential { value, frames }
}

/// A wrapper for a type that is interleaved.
pub struct Interleaved<T> {
    value: T,
    channels: usize,
}

impl<T> BufInfo for Interleaved<T>
where
    T: Slice,
{
    fn buf_info_frames(&self) -> usize {
        self.value.slice_len() / self.channels
    }

    fn buf_info_channels(&self) -> usize {
        self.channels
    }
}

impl<T, S> Buf<S> for Interleaved<T>
where
    T: AsRef<[S]>,
    T: Slice,
    S: Sample,
{
    fn channel(&self, channel: usize) -> Channel<'_, S> {
        if self.channels == 1 && channel == 0 {
            Channel::linear(self.value.as_ref())
        } else {
            Channel::interleaved(self.value.as_ref(), self.channels, channel)
        }
    }
}

impl<T> ResizableBuf for Interleaved<T>
where
    T: Slice,
{
    fn resize(&mut self, frames: usize) {
        if self.value.slice_len() / self.channels != frames {
            panic!("buffer cannot be resized")
        }
    }

    fn resize_topology(&mut self, channels: usize, frames: usize) {
        if self.channels != channels || self.value.slice_len() / self.channels != frames {
            panic!("buffer cannot be resized")
        }
    }
}

impl<T, S> BufMut<S> for Interleaved<T>
where
    T: Slice + AsRef<[S]> + AsMut<[S]>,
    S: Sample,
{
    fn channel_mut(&mut self, channel: usize) -> ChannelMut<'_, S> {
        if self.channels == 1 && channel == 0 {
            ChannelMut::linear(self.value.as_mut())
        } else {
            ChannelMut::interleaved(self.value.as_mut(), self.channels, channel)
        }
    }
}

impl<T> ReadBuf for Interleaved<&'_ [T]> {
    fn remaining(&self) -> usize {
        self.buf_info_frames()
    }

    fn advance(&mut self, n: usize) {
        let end = usize::min(self.value.len(), n);
        self.value = &self.value[end..];
    }
}

impl<T> ReadBuf for Interleaved<&'_ mut [T]> {
    fn remaining(&self) -> usize {
        self.buf_info_frames()
    }

    fn advance(&mut self, n: usize) {
        let value = std::mem::take(&mut self.value);
        let end = usize::min(value.len(), n);
        self.value = &mut value[end..];
    }
}

/// A wrapper for a type that is interleaved.
pub struct Sequential<T> {
    value: T,
    frames: usize,
}

impl<T> BufInfo for Sequential<T>
where
    T: Slice,
{
    fn buf_info_frames(&self) -> usize {
        self.frames
    }

    fn buf_info_channels(&self) -> usize {
        self.value.slice_len() / self.frames
    }
}

impl<T, S> Buf<S> for Sequential<T>
where
    T: Slice + AsRef<[S]>,
    S: Sample,
{
    fn channel(&self, channel: usize) -> Channel<'_, S> {
        let value = self.value.as_ref();
        let value = &value[channel * self.frames..];
        let value = &value[..self.frames];

        Channel::linear(value)
    }
}

impl<T> ResizableBuf for Sequential<T>
where
    T: Slice,
{
    fn resize(&mut self, frames: usize) {
        if self.frames != frames {
            panic!("buffer cannot be resized")
        }
    }

    fn resize_topology(&mut self, channels: usize, frames: usize) {
        if self.frames != frames || self.value.slice_len() / self.frames != channels {
            panic!("buffer cannot be resized")
        }
    }
}

impl<T, S> BufMut<S> for Sequential<T>
where
    T: Slice + AsRef<[S]> + AsMut<[S]>,
    S: Sample,
{
    fn channel_mut(&mut self, channel: usize) -> ChannelMut<'_, S> {
        let value = self.value.as_mut();
        let value = &mut value[channel * self.frames..];
        let value = &mut value[..self.frames];

        ChannelMut::linear(value)
    }
}
