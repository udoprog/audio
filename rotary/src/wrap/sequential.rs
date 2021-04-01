use rotary_core::{Buf, BufMut, ExactSizeBuf};
use rotary_core::{Channel, ChannelMut};

/// A wrapper for a sequential audio buffer.
///
/// See [wrap::sequential][super::sequential()].
pub struct Sequential<T> {
    value: T,
    frames: usize,
}

impl<T> Sequential<T> {
    pub(super) fn new(value: T, frames: usize) -> Self {
        Self { value, frames }
    }

    /// Convert back into the wrapped value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let buffer = rotary::wrap::sequential(&[1, 2, 3, 4], 2);
    /// assert_eq!(buffer.into_inner(), &[1, 2, 3, 4]);
    /// ```
    pub fn into_inner(self) -> T {
        self.value
    }
}

macro_rules! impl_buf {
    ([$($p:tt)*] , $ty:ty $(, $len:ident)?) => {
        impl<$($p)*> ExactSizeBuf for Sequential<$ty> {
            fn frames(&self) -> usize {
                self.frames
            }
        }

        impl<$($p)*> Buf<T> for Sequential<$ty> {
            fn frames_hint(&self) -> Option<usize> {
                Some(self.frames)
            }

            fn channels(&self) -> usize {
                impl_buf!(@frames self, $($len)*) / self.frames
            }

            fn channel(&self, channel: usize) -> Channel<'_, T> {
                let value = &self.value[channel * self.frames..];
                let value = &value[..self.frames];
                Channel::linear(value)
            }
        }
    };

    (@frames $s:ident,) => { $s.value.len() };
    (@frames $_:ident, $n:ident) => { $n };
}

impl_buf!([T], &'_ [T]);
impl_buf!([T], &'_ mut [T]);
impl_buf!([T, const N: usize], [T; N], N);
impl_buf!([T, const N: usize], &'_ [T; N], N);
impl_buf!([T, const N: usize], &'_ mut [T; N], N);

macro_rules! impl_buf_mut {
    ([$($p:tt)*], $ty:ty) => {
        impl<$($p)*> BufMut<T> for Sequential<$ty> {
            fn channel_mut(&mut self, channel: usize) -> ChannelMut<'_, T> {
                let value = &mut self.value[channel * self.frames..];
                let value = &mut value[..self.frames];

                ChannelMut::linear(value)
            }
        }
    };
}

impl_buf_mut!([T], &'_ mut [T]);
impl_buf_mut!([T, const N: usize], &'_ mut [T; N]);
