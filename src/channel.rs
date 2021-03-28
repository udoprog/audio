//! A reference to a channel in a buffer.

use crate::sample::Sample;
use std::cmp;
use std::fmt;
use std::hash;
use std::marker;

/// A reference to a channel in a buffer.
///
/// See [crate::Interleaved::get].
#[derive(Clone, Copy)]
pub struct Channel<'a, T>
where
    T: Sample,
{
    pub(crate) buffer: *const T,
    pub(crate) channel: usize,
    pub(crate) channels: usize,
    pub(crate) frames: usize,
    pub(crate) _marker: marker::PhantomData<&'a T>,
}

unsafe impl<T> Send for Channel<'_, T> where T: Sample + Send {}
unsafe impl<T> Sync for Channel<'_, T> where T: Sample + Sync {}

impl<'a, T> Channel<'a, T>
where
    T: Sample,
{
    /// Get a reference to a frame.
    pub fn get(&self, frame: usize) -> Option<&T> {
        if frame < self.frames {
            let offset = self.channel * self.frames + frame;
            let frame = unsafe { &*self.buffer.add(offset) };
            Some(frame)
        } else {
            None
        }
    }

    /// Construct an iterator over the current channel.
    pub fn iter(&self) -> ChannelIter<'_, T> {
        ChannelIter {
            buffer: self.buffer,
            frame: 0,
            channel: self.channel,
            channels: self.channels,
            frames: self.frames,
            _marker: marker::PhantomData,
        }
    }
}

impl<T> fmt::Debug for Channel<'_, T>
where
    T: Sample + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T> cmp::PartialEq for Channel<'_, T>
where
    T: Sample + cmp::PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<T> cmp::Eq for Channel<'_, T> where T: Sample + cmp::Eq {}

impl<T> cmp::PartialOrd for Channel<'_, T>
where
    T: Sample + cmp::PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.iter().partial_cmp(other.iter())
    }
}

impl<T> cmp::Ord for Channel<'_, T>
where
    T: Sample + cmp::Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.iter().cmp(other.iter())
    }
}

impl<T> hash::Hash for Channel<'_, T>
where
    T: Sample + hash::Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        for frame in self.iter() {
            frame.hash(state);
        }
    }
}

/// An iterator over a channel.
///
/// Created with [Channel::iter].
pub struct ChannelIter<'a, T>
where
    T: Sample,
{
    buffer: *const T,
    frame: usize,
    channel: usize,
    channels: usize,
    frames: usize,
    _marker: marker::PhantomData<&'a T>,
}

unsafe impl<T> Send for ChannelIter<'_, T> where T: Sample + Send {}
unsafe impl<T> Sync for ChannelIter<'_, T> where T: Sample + Sync {}

impl<'a, T> Iterator for ChannelIter<'a, T>
where
    T: Sample,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.frame < self.frames {
            let offset = self.frame * self.channels + self.channel;
            let frame = unsafe { &*self.buffer.add(offset) };
            self.frame += 1;
            Some(frame)
        } else {
            None
        }
    }
}

/// A mutable reference to a channel in a buffer.
///
/// See [crate::Interleaved::get_mut].
pub struct ChannelMut<'a, T>
where
    T: Sample,
{
    pub(crate) buffer: *mut T,
    pub(crate) channel: usize,
    pub(crate) channels: usize,
    pub(crate) frames: usize,
    pub(crate) _marker: marker::PhantomData<&'a mut T>,
}

unsafe impl<T> Send for ChannelMut<'_, T> where T: Sample + Send {}
unsafe impl<T> Sync for ChannelMut<'_, T> where T: Sample + Sync {}

impl<'a, T> ChannelMut<'a, T>
where
    T: Sample,
{
    /// Get a reference to a frame.
    pub fn get(&self, frame: usize) -> Option<&T> {
        if frame < self.frames {
            let offset = self.channel * self.frames + frame;
            let frame = unsafe { &*self.buffer.add(offset) };
            Some(frame)
        } else {
            None
        }
    }

    /// Get a mutable reference to a frame.
    pub fn get_mut(&mut self, frame: usize) -> Option<&mut T> {
        if frame < self.frames {
            let offset = self.channel * self.frames + frame;
            let frame = unsafe { &mut *self.buffer.add(offset) };
            Some(frame)
        } else {
            None
        }
    }

    /// Construct an iterator over the current channel.
    pub fn iter(&self) -> ChannelIter<'_, T> {
        ChannelIter {
            buffer: self.buffer as *const T,
            frame: 0,
            channel: self.channel,
            channels: self.channels,
            frames: self.frames,
            _marker: marker::PhantomData,
        }
    }

    /// Construct a mutable iterator over the current channel.
    pub fn iter_mut(&mut self) -> ChannelIterMut<'_, T> {
        ChannelIterMut {
            buffer: self.buffer,
            frame: 0,
            channel: self.channel,
            channels: self.channels,
            frames: self.frames,
            _marker: marker::PhantomData,
        }
    }
}

impl<T> fmt::Debug for ChannelMut<'_, T>
where
    T: Sample + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T> cmp::PartialEq for ChannelMut<'_, T>
where
    T: Sample + cmp::PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<T> cmp::Eq for ChannelMut<'_, T> where T: Sample + cmp::Eq {}

impl<T> cmp::PartialOrd for ChannelMut<'_, T>
where
    T: Sample + cmp::PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.iter().partial_cmp(other.iter())
    }
}

impl<T> cmp::Ord for ChannelMut<'_, T>
where
    T: Sample + cmp::Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.iter().cmp(other.iter())
    }
}

impl<T> hash::Hash for ChannelMut<'_, T>
where
    T: Sample + hash::Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        for frame in self.iter() {
            frame.hash(state);
        }
    }
}

/// A mutable iterator over a channel.
///
/// Created with [ChannelMut::iter_mut].
pub struct ChannelIterMut<'a, T>
where
    T: Sample,
{
    buffer: *mut T,
    frame: usize,
    channel: usize,
    channels: usize,
    frames: usize,
    _marker: marker::PhantomData<&'a T>,
}

unsafe impl<T> Send for ChannelIterMut<'_, T> where T: Sample + Send {}
unsafe impl<T> Sync for ChannelIterMut<'_, T> where T: Sample + Sync {}

impl<'a, T> Iterator for ChannelIterMut<'a, T>
where
    T: Sample,
{
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.frame < self.frames {
            let offset = self.frame * self.channels + self.channel;
            let frame = unsafe { &mut *self.buffer.add(offset) };
            self.frame += 1;
            Some(frame)
        } else {
            None
        }
    }
}
