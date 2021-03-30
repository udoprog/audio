//! Wrap an external type to implement [Buf] and [BufMut].

use crate::buf::{Buf, BufInfo, BufMut};
use crate::channel::{Channel, ChannelMut};
use crate::io::{ReadBuf, WriteBuf};
use crate::sample::Sample;
use crate::translate::Translate;

/// Wrap a `value` as an interleaved buffer with the given number of channels.
///
/// An interleaved buffer is a bit special in that it can implement [ReadBuf]
/// and [WriteBuf] directly if it wraps one of the following types:
/// * `&[T]` - Will implement [ReadBuf].
/// * `&mut [T]` - Will implement [WriteBuf].
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

impl<T> BufInfo for Interleaved<&'_ [T]> {
    fn buf_info_frames(&self) -> usize {
        self.value.len() / self.channels
    }

    fn buf_info_channels(&self) -> usize {
        self.channels
    }
}

impl<T> BufInfo for Interleaved<&'_ mut [T]> {
    fn buf_info_frames(&self) -> usize {
        self.value.len() / self.channels
    }

    fn buf_info_channels(&self) -> usize {
        self.channels
    }
}

impl<T> Buf<T> for Interleaved<&'_ [T]>
where
    T: Sample,
{
    fn channel(&self, channel: usize) -> Channel<'_, T> {
        if self.channels == 1 && channel == 0 {
            Channel::linear(self.value.as_ref())
        } else {
            Channel::interleaved(self.value.as_ref(), self.channels, channel)
        }
    }
}

impl<T> Buf<T> for Interleaved<&'_ mut [T]>
where
    T: Sample,
{
    fn channel(&self, channel: usize) -> Channel<'_, T> {
        if self.channels == 1 && channel == 0 {
            Channel::linear(self.value.as_ref())
        } else {
            Channel::interleaved(self.value.as_ref(), self.channels, channel)
        }
    }
}

impl<T> BufMut<T> for Interleaved<&'_ mut [T]>
where
    T: Sample,
{
    fn channel_mut(&mut self, channel: usize) -> ChannelMut<'_, T> {
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
        self.value = &self.value[n * self.channels..];
    }
}

impl<T> WriteBuf<T> for Interleaved<&'_ mut [T]>
where
    T: Sample,
{
    fn remaining_mut(&self) -> usize {
        self.buf_info_frames()
    }

    fn copy<I>(&mut self, mut buf: I)
    where
        I: ReadBuf + Buf<T>,
    {
        let len = usize::min(self.remaining_mut(), buf.remaining());
        crate::utils::copy(&buf, &mut *self);
        let end = usize::min(self.value.len(), len * self.channels);
        let value = std::mem::take(&mut self.value);
        self.value = &mut value[end..];
        buf.advance(len);
    }

    fn translate<I, U>(&mut self, mut buf: I)
    where
        T: Translate<U>,
        I: ReadBuf + Buf<U>,
        U: Sample,
    {
        let len = usize::min(self.remaining_mut(), buf.remaining());
        crate::utils::translate(&buf, &mut *self);
        let end = usize::min(self.value.len(), len * self.channels);
        let value = std::mem::take(&mut self.value);
        self.value = &mut value[end..];
        buf.advance(len);
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

impl<T> BufInfo for Sequential<&'_ [T]> {
    fn buf_info_frames(&self) -> usize {
        self.frames
    }

    fn buf_info_channels(&self) -> usize {
        self.value.len() / self.frames
    }
}

impl<T> BufInfo for Sequential<&'_ mut [T]> {
    fn buf_info_frames(&self) -> usize {
        self.frames
    }

    fn buf_info_channels(&self) -> usize {
        self.value.len() / self.frames
    }
}

impl<T> Buf<T> for Sequential<&'_ [T]>
where
    T: Sample,
{
    fn channel(&self, channel: usize) -> Channel<'_, T> {
        let value = &self.value[channel * self.frames..];
        let value = &value[..self.frames];

        Channel::linear(value)
    }
}

impl<T> Buf<T> for Sequential<&'_ mut [T]>
where
    T: Sample,
{
    fn channel(&self, channel: usize) -> Channel<'_, T> {
        let value = &self.value[channel * self.frames..];
        let value = &value[..self.frames];

        Channel::linear(value)
    }
}

impl<T> BufMut<T> for Sequential<&'_ mut [T]>
where
    T: Sample,
{
    fn channel_mut(&mut self, channel: usize) -> ChannelMut<'_, T> {
        let value = &mut self.value[channel * self.frames..];
        let value = &mut value[..self.frames];

        ChannelMut::linear(value)
    }
}
