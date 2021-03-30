use crate::interleaved::channel::{Channel, ChannelMut, RawChannelMut, RawChannelRef};
use crate::sample::Sample;
use std::marker;

/// An iterator over the channels in the buffer.
///
/// Created with [Interleaved::iter][super::Interleaved::iter].
pub struct Iter<'a, T>
where
    T: Sample,
{
    pub(super) buffer: *const T,
    pub(super) channel: usize,
    pub(super) channels: usize,
    pub(super) frames: usize,
    pub(super) _marker: marker::PhantomData<&'a T>,
}

// Safety: the iterator is simply a container of T's, any Send/Sync properties
// are inherited.
unsafe impl<T> Send for Iter<'_, T> where T: Sample + Send {}
unsafe impl<T> Sync for Iter<'_, T> where T: Sample + Sync {}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: Sample,
{
    type Item = Channel<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.channel < self.channels {
            let channel = self.channel;
            self.channel += 1;

            Some(Channel {
                inner: RawChannelRef {
                    buffer: self.buffer,
                    channel,
                    frames: self.frames,
                    channels: self.channels,
                },
                _marker: marker::PhantomData,
            })
        } else {
            None
        }
    }
}

/// A mutable iterator over the channels in the buffer.
///
/// Created with [Interleaved::iter_mut][super::Interleaved::iter_mut].
pub struct IterMut<'a, T>
where
    T: Sample,
{
    pub(super) buffer: *mut T,
    pub(super) channel: usize,
    pub(super) channels: usize,
    pub(super) frames: usize,
    pub(super) _marker: marker::PhantomData<&'a mut T>,
}

// Safety: the iterator is simply a container of T's, any Send/Sync properties
// are inherited.
unsafe impl<T> Send for IterMut<'_, T> where T: Sample + Send {}
unsafe impl<T> Sync for IterMut<'_, T> where T: Sample + Sync {}

impl<'a, T> Iterator for IterMut<'a, T>
where
    T: Sample,
{
    type Item = ChannelMut<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.channel < self.channels {
            let channel = self.channel;
            self.channel += 1;

            Some(ChannelMut {
                inner: RawChannelMut {
                    buffer: self.buffer,
                    channel,
                    frames: self.frames,
                    channels: self.channels,
                },
                _marker: marker::PhantomData,
            })
        } else {
            None
        }
    }
}
