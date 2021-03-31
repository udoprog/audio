use rotary_core::io::{ReadBuf, WriteBuf};
use rotary_core::{Buf, BufInfo, BufMut, Channel, ChannelMut, Translate};

/// A wrapper for a type that is interleaved.
pub struct Interleaved<T> {
    value: T,
    channels: usize,
}

impl<T> Interleaved<T> {
    pub(super) fn new(value: T, channels: usize) -> Self {
        Self { value, channels }
    }
}

impl<T> BufInfo for Interleaved<&'_ [T]> {
    fn frames(&self) -> usize {
        self.value.len() / self.channels
    }

    fn channels(&self) -> usize {
        self.channels
    }
}

impl<T> BufInfo for Interleaved<&'_ mut [T]> {
    fn frames(&self) -> usize {
        self.value.len() / self.channels
    }

    fn channels(&self) -> usize {
        self.channels
    }
}

impl<T> Buf<T> for Interleaved<&'_ [T]> {
    fn channel(&self, channel: usize) -> Channel<'_, T> {
        if self.channels == 1 && channel == 0 {
            Channel::linear(self.value.as_ref())
        } else {
            Channel::interleaved(self.value.as_ref(), self.channels, channel)
        }
    }
}

impl<T> Buf<T> for Interleaved<&'_ mut [T]> {
    fn channel(&self, channel: usize) -> Channel<'_, T> {
        if self.channels == 1 && channel == 0 {
            Channel::linear(self.value.as_ref())
        } else {
            Channel::interleaved(self.value.as_ref(), self.channels, channel)
        }
    }
}

impl<T> BufMut<T> for Interleaved<&'_ mut [T]> {
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
        self.frames()
    }

    fn advance(&mut self, n: usize) {
        self.value = &self.value[n * self.channels..];
    }
}

impl<T> WriteBuf<T> for Interleaved<&'_ mut [T]> {
    fn remaining_mut(&self) -> usize {
        self.frames()
    }

    fn copy<I>(&mut self, mut buf: I)
    where
        I: ReadBuf + Buf<T>,
        T: Copy,
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
        U: Copy,
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
        self.frames()
    }

    fn advance(&mut self, n: usize) {
        let value = std::mem::take(&mut self.value);
        let end = usize::min(value.len(), n);
        self.value = &mut value[end..];
    }
}
