use rotary_core::{Buf, BufMut, Channel, ChannelMut, ExactSizeBuf, ReadBuf, WriteBuf};

/// A wrapper for an interleaved audio buffer.
///
/// See [wrap::interleaved][super::interleaved()].
pub struct Interleaved<T> {
    value: T,
    channels: usize,
}

impl<T> Interleaved<T> {
    pub(super) fn new(value: T, channels: usize) -> Self {
        Self { value, channels }
    }
}

impl<T, const N: usize> ExactSizeBuf for Interleaved<&'_ [T; N]> {
    fn frames(&self) -> usize {
        N / self.channels
    }
}

impl<T, const N: usize> ExactSizeBuf for Interleaved<&'_ mut [T; N]> {
    fn frames(&self) -> usize {
        N / self.channels
    }
}

impl<T, const N: usize> ExactSizeBuf for Interleaved<[T; N]> {
    fn frames(&self) -> usize {
        N / self.channels
    }
}

impl<T> ExactSizeBuf for Interleaved<&'_ [T]> {
    fn frames(&self) -> usize {
        self.value.len() / self.channels
    }
}

impl<T> ExactSizeBuf for Interleaved<&'_ mut [T]> {
    fn frames(&self) -> usize {
        self.value.len() / self.channels
    }
}

macro_rules! impl_buf {
    ([$($p:tt)*], $ty:ty) => {
        impl<$($p)*> Buf<T> for Interleaved<$ty> {
            fn frames_hint(&self) -> Option<usize> {
                Some(self.frames())
            }

            fn channels(&self) -> usize {
                self.channels
            }

            fn channel(&self, channel: usize) -> Channel<'_, T> {
                if self.channels == 1 && channel == 0 {
                    Channel::linear(self.value.as_ref())
                } else {
                    Channel::interleaved(self.value.as_ref(), self.channels, channel)
                }
            }
        }
    };
}

impl_buf!([T], &'_ [T]);
impl_buf!([T], &'_ mut [T]);
impl_buf!([T, const N: usize], [T; N]);
impl_buf!([T, const N: usize], &'_ [T; N]);
impl_buf!([T, const N: usize], &'_ mut [T; N]);

macro_rules! impl_buf_mut {
    ([$($p:tt)*], $ty:ty) => {
        impl<$($p)*> BufMut<T> for Interleaved<$ty> {
            fn channel_mut(&mut self, channel: usize) -> ChannelMut<'_, T> {
                if self.channels == 1 && channel == 0 {
                    ChannelMut::linear(self.value.as_mut())
                } else {
                    ChannelMut::interleaved(self.value.as_mut(), self.channels, channel)
                }
            }
        }
    };
}

impl_buf_mut!([T], &'_ mut [T]);
impl_buf_mut!([T, const N: usize], &'_ mut [T; N]);

impl<T> ReadBuf for Interleaved<&'_ [T]> {
    fn remaining(&self) -> usize {
        self.frames()
    }

    fn advance(&mut self, n: usize) {
        self.value = self
            .value
            .get(n.saturating_mul(self.channels)..)
            .unwrap_or_default();
    }
}

impl<T> ReadBuf for Interleaved<&'_ mut [T]> {
    fn remaining(&self) -> usize {
        self.frames()
    }

    fn advance(&mut self, n: usize) {
        let value = std::mem::take(&mut self.value);
        self.value = value
            .get_mut(n.saturating_mul(self.channels)..)
            .unwrap_or_default();
    }
}

impl<T> WriteBuf for Interleaved<&'_ mut [T]> {
    fn remaining_mut(&self) -> usize {
        self.frames()
    }

    fn advance_mut(&mut self, n: usize) {
        let value = std::mem::take(&mut self.value);
        self.value = value
            .get_mut(n.saturating_mul(self.channels)..)
            .unwrap_or_default();
    }
}
