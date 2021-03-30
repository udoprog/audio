use crate::buf::{Buf, BufInfo, BufMut};
use crate::channel::{Channel, ChannelMut};
use crate::sample::Sample;

/// A wrapper for a type that is interleaved.
pub struct Sequential<T> {
    value: T,
    frames: usize,
}

impl<T> Sequential<T> {
    pub(super) fn new(value: T, frames: usize) -> Self {
        Self { value, frames }
    }
}

impl<T> BufInfo for Sequential<&'_ [T]>
where
    T: Sample,
{
    fn buf_info_frames(&self) -> usize {
        self.frames
    }

    fn buf_info_channels(&self) -> usize {
        self.value.len() / self.frames
    }
}

impl<T> BufInfo for Sequential<&'_ mut [T]>
where
    T: Sample,
{
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
