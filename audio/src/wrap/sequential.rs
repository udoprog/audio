use audio_core::{Buf, Channels, ChannelsMut, ExactSizeBuf, LinearChannel, LinearChannelMut};

/// A wrapper for a sequential audio buffer.
///
/// See [wrap::sequential][super::sequential()].
pub struct Sequential<T> {
    value: T,
    channels: usize,
}

impl<T> Sequential<T> {
    pub(super) fn new(value: T, channels: usize) -> Self {
        Self { value, channels }
    }

    /// Convert back into the wrapped value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let buffer = audio::wrap::sequential(&[1, 2, 3, 4], 2);
    /// assert_eq!(buffer.into_inner(), &[1, 2, 3, 4]);
    /// ```
    pub fn into_inner(self) -> T {
        self.value
    }
}

macro_rules! impl_buf {
    ([$($p:tt)*] , $ty:ty $(, $len:ident)?) => {
        impl<$($p)*> Buf for Sequential<$ty> {
            fn frames_hint(&self) -> Option<usize> {
                Some(self.frames())
            }

            fn channels(&self) -> usize {
                self.channels
            }
        }

        impl<$($p)*> ExactSizeBuf for Sequential<$ty> {
            fn frames(&self) -> usize {
                impl_buf!(@frames self, $($len)*) / self.channels
            }
        }

        impl<$($p)*> Channels<T> for Sequential<$ty> {
            type Channel<'a> where T: 'a = LinearChannel<'a, T>;

            fn channel(&self, channel: usize) -> Self::Channel<'_> {
                let frames = self.frames();
                let value = self.value.get(channel * frames..).unwrap_or_default();
                let value = value.get(..frames).unwrap_or_default();
                LinearChannel::new(value)
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
        impl<$($p)*> ChannelsMut<T> for Sequential<$ty> where T: Copy {
            type ChannelMut<'a> where T: 'a = LinearChannelMut<'a, T>;

            fn channel_mut(&mut self, channel: usize) -> Self::ChannelMut<'_> {
                let frames = self.frames();
                let value = self.value.get_mut(channel * frames..).unwrap_or_default();
                let value = value.get_mut(..frames).unwrap_or_default();
                LinearChannelMut::new(value)
            }

            fn copy_channels(&mut self, from: usize, to: usize) {
                let frames = self.frames();

                // Safety: We're calling the copy function with internal
                // parameters which are guaranteed to be correct. `channels` is
                // guaranteed to reflect a valid subset of the buffer based on
                // frames, because it uses the trusted length of the provided
                // slice.
                unsafe {
                    crate::utils::copy_channels_sequential(
                        self.value.as_mut_ptr(),
                        self.channels,
                        frames,
                        from,
                        to,
                    );
                }
            }
        }
    };
}

impl_buf_mut!([T], &'_ mut [T]);
impl_buf_mut!([T, const N: usize], &'_ mut [T; N]);
