use audio_core::{
    AsInterleaved, AsInterleavedMut, Buf, BufMut, ExactSizeBuf, InterleavedBuf, InterleavedChannel,
    InterleavedChannelMut, ReadBuf, WriteBuf,
};

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

    /// Convert back into the wrapped value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let buffer = audio::wrap::interleaved(&[1, 2, 3, 4], 2);
    /// assert_eq!(buffer.into_inner(), &[1, 2, 3, 4]);
    /// ```
    pub fn into_inner(self) -> T {
        self.value
    }
}

macro_rules! impl_buf {
    ([$($p:tt)*], $ty:ty $(, $len:ident)?) => {
        impl<$($p)*> Buf for Interleaved<$ty> {
            type Sample = T;
            type Channel<'a> where Self::Sample: 'a = InterleavedChannel<'a, Self::Sample>;

            fn frames_hint(&self) -> Option<usize> {
                Some(self.frames())
            }

            fn channels(&self) -> usize {
                self.channels
            }

            fn channel(&self, channel: usize) -> Self::Channel<'_> {
                InterleavedChannel::new(self.value.as_ref(), self.channels, channel)
            }
        }

        impl<$($p)*> ExactSizeBuf for Interleaved<$ty> {
            fn frames(&self) -> usize {
                impl_buf!(@frames self, $($len)*) / self.channels
            }
        }

        impl<$($p)*> AsInterleaved<T> for Interleaved<$ty> {
            fn as_interleaved(&self) -> &[T] {
                self.value.as_ref()
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
        impl<$($p)*> BufMut for Interleaved<$ty> where T: Copy {
            type ChannelMut<'a> where Self::Sample: 'a = InterleavedChannelMut<'a, Self::Sample>;

            fn channel_mut(&mut self, channel: usize) -> Self::ChannelMut<'_> {
                InterleavedChannelMut::new(self.value.as_mut(), self.channels, channel)
            }

            fn copy_channels(&mut self, from: usize, to: usize) {
                let frames = self.frames();

                // Safety: We're calling the copy function with internal
                // parameters which are guaranteed to be correct. `frames` is
                // guaranteed to reflect a valid subset of the buffer based on
                // frames, because it uses the trusted length of the provided
                // slice.
                unsafe {
                    crate::utils::copy_channels_interleaved(
                        self.value.as_mut_ptr(),
                        self.channels,
                        frames,
                        from,
                        to,
                    );
                }
            }
        }

        impl<$($p)*> AsInterleavedMut<T> for Interleaved<$ty> {
            fn as_interleaved_mut(&mut self) -> &mut [T] {
                self.value
            }

            fn as_interleaved_mut_ptr(&mut self) -> *mut T {
                self.value.as_mut_ptr()
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

impl<T> InterleavedBuf for Interleaved<&'_ mut [T]> {
    fn reserve_frames(&mut self, frames: usize) {
        if frames > self.value.len() {
            panic!(
                "required number of frames {new_len} is larger than the wrapped buffer {len}",
                new_len = frames,
                len = self.value.len()
            );
        }
    }

    fn set_topology(&mut self, channels: usize, frames: usize) {
        let new_len = channels * frames;
        let len = self.value.len();

        let value = std::mem::take(&mut self.value);

        let value = match value.get_mut(..new_len) {
            Some(value) => value,
            None => {
                panic!(
                    "the topology {channels}:{frames} requires {new_len}, which is larger than the wrapped buffer {len}",
                    channels = channels,
                    frames = frames,
                    new_len = new_len,
                    len = len,
                );
            }
        };

        self.value = value;
        self.channels = channels;
    }
}
