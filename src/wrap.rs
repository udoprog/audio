//! Wrap an external type to implement [Buf] and [BufMut].

use crate::buf::{Buf, BufChannel, BufChannelMut, BufMut};

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

impl<T, S> Buf<S> for Interleaved<T>
where
    T: AsRef<[S]>,
{
    fn frames(&self) -> usize {
        self.value.as_ref().len() / self.channels
    }

    fn channels(&self) -> usize {
        self.channels
    }

    fn channel(&self, channel: usize) -> BufChannel<'_, S> {
        if self.channels == 1 && channel == 0 {
            BufChannel::linear(self.value.as_ref())
        } else {
            BufChannel::interleaved(self.value.as_ref(), self.channels, channel)
        }
    }
}

impl<T, S> BufMut<S> for Interleaved<T>
where
    T: AsRef<[S]> + AsMut<[S]>,
{
    fn channel_mut(&mut self, channel: usize) -> BufChannelMut<'_, S> {
        if self.channels == 1 && channel == 0 {
            BufChannelMut::linear(self.value.as_mut())
        } else {
            BufChannelMut::interleaved(self.value.as_mut(), self.channels, channel)
        }
    }

    fn resize(&mut self, frames: usize) {
        if self.value.as_ref().len() / self.channels != frames {
            panic!("buffer cannot be resized")
        }
    }

    fn resize_topology(&mut self, channels: usize, frames: usize) {
        if self.channels != channels || self.value.as_ref().len() / self.channels != frames {
            panic!("buffer cannot be resized")
        }
    }
}

/// A wrapper for a type that is interleaved.
pub struct Sequential<T> {
    value: T,
    frames: usize,
}

impl<T, S> Buf<S> for Sequential<T>
where
    T: AsRef<[S]>,
{
    fn frames(&self) -> usize {
        self.frames
    }

    fn channels(&self) -> usize {
        self.value.as_ref().len() / self.frames
    }

    fn channel(&self, channel: usize) -> BufChannel<'_, S> {
        let value = self.value.as_ref();
        let value = &value[channel * self.frames..];
        let value = &value[..self.frames];

        BufChannel::linear(value)
    }
}

impl<T, S> BufMut<S> for Sequential<T>
where
    T: AsRef<[S]> + AsMut<[S]>,
{
    fn channel_mut(&mut self, channel: usize) -> BufChannelMut<'_, S> {
        let value = self.value.as_mut();
        let value = &mut value[channel * self.frames..];
        let value = &mut value[..self.frames];

        BufChannelMut::linear(value)
    }

    fn resize(&mut self, frames: usize) {
        if self.frames != frames {
            panic!("buffer cannot be resized")
        }
    }

    fn resize_topology(&mut self, channels: usize, frames: usize) {
        if self.frames != frames || self.value.as_ref().len() / self.frames != channels {
            panic!("buffer cannot be resized")
        }
    }
}
