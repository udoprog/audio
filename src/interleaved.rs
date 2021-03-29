//! A dynamically sized, multi-channel interleaved audio buffer.

use crate::buf::{Buf, BufChannel, BufChannelMut, BufMut};
use crate::channel::{Channel, ChannelMut, RawChannelMut, RawChannelRef};
use crate::sample::Sample;
use crate::wrap;
use std::cmp;
use std::fmt;
use std::hash;
use std::marker;
use std::ptr;

/// A dynamically sized, multi-channel interleaved audio buffer.
///
/// An audio buffer is constrained to only support sample-apt types. For more
/// information of what this means, see [Sample].
///
/// An *interleaved* audio buffer stores all audio data interleaved in memory,
/// one sample from each channel in sequence until we're out of samples. This
/// naturally makes the buffer a bit harder to work with, and we have to rely on
/// iterators to access logical channels.
///
/// Resized regions aren't zeroed, so certain operations might cause stale data
/// to be visible after a resize.
///
/// ```rust
/// let mut buffer = rotary::Interleaved::<f32>::with_topology(2, 4);
///
/// for (c, s) in buffer
///     .get_mut(0)
///     .unwrap()
///     .iter_mut()
///     .zip(&[1.0, 2.0, 3.0, 4.0])
/// {
///     *c = *s;
/// }
///
/// for (c, s) in buffer
///     .get_mut(1)
///     .unwrap()
///     .iter_mut()
///     .zip(&[5.0, 6.0, 7.0, 8.0])
/// {
///     *c = *s;
/// }
///
/// assert_eq!(buffer.as_slice(), &[1.0, 5.0, 2.0, 6.0, 3.0, 7.0, 4.0, 8.0]);
/// ```
pub struct Interleaved<T>
where
    T: Sample,
{
    data: Vec<T>,
    channels: usize,
    frames: usize,
}

impl<T> Interleaved<T>
where
    T: Sample,
{
    /// Construct a new empty audio buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = rotary::Interleaved::<f32>::new();
    ///
    /// assert_eq!(buffer.frames(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            channels: 0,
            frames: 0,
        }
    }

    /// Allocate an audio buffer with the given topology. A "topology" is a
    /// given number of `channels` and the corresponding number of `frames` in
    /// their buffers.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = rotary::Interleaved::<f32>::with_topology(4, 256);
    ///
    /// assert_eq!(buffer.frames(), 256);
    /// assert_eq!(buffer.channels(), 4);
    /// ```
    pub fn with_topology(channels: usize, frames: usize) -> Self {
        Self {
            data: vec![T::ZERO; channels * frames],
            channels,
            frames,
        }
    }

    /// Allocate an audio buffer from a fixed-size array.
    ///
    /// See [dynamic!].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::BitSet;
    ///
    /// let mut buffer = rotary::interleaved![[2.0; 256]; 4];
    ///
    /// assert_eq!(buffer.frames(), 256);
    /// assert_eq!(buffer.channels(), 4);
    ///
    /// for chan in &buffer {
    ///     assert!(chan.iter().eq(&[2.0; 256][..]));
    /// }
    /// ```
    pub fn from_vec(data: Vec<T>, channels: usize, frames: usize) -> Self {
        Self {
            data,
            channels,
            frames,
        }
    }

    /// Take ownership of the backing vector.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = rotary::Interleaved::<f32>::with_topology(2, 4);
    ///
    /// for (c, s) in buffer.get_mut(0).unwrap().iter_mut().zip(&[1.0, 2.0, 3.0, 4.0]) {
    ///     *c = *s;
    /// }
    ///
    /// for (c, s) in buffer.get_mut(1).unwrap().iter_mut().zip(&[1.0, 2.0, 3.0, 4.0]) {
    ///     *c = *s;
    /// }
    ///
    /// buffer.resize(3);
    ///
    /// assert_eq!(buffer.into_vec(), vec![1.0, 1.0, 2.0, 2.0, 3.0, 3.0])
    /// ```
    pub fn into_vec(self) -> Vec<T> {
        self.data
    }

    /// Access the underlying vector as a slice.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = rotary::Interleaved::<f32>::with_topology(2, 4);
    ///
    /// for (c, s) in buffer.get_mut(0).unwrap().iter_mut().zip(&[1.0, 2.0, 3.0, 4.0]) {
    ///     *c = *s;
    /// }
    ///
    /// for (c, s) in buffer.get_mut(1).unwrap().iter_mut().zip(&[1.0, 2.0, 3.0, 4.0]) {
    ///     *c = *s;
    /// }
    ///
    /// buffer.resize(3);
    ///
    /// assert_eq!(buffer.as_slice(), &[1.0, 1.0, 2.0, 2.0, 3.0, 3.0])
    /// ```
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    /// Get the number of frames in the channels of an audio buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = rotary::Interleaved::<f32>::new();
    ///
    /// assert_eq!(buffer.frames(), 0);
    /// buffer.resize(256);
    /// assert_eq!(buffer.frames(), 256);
    /// ```
    pub fn frames(&self) -> usize {
        self.frames
    }

    /// Check how many channels there are in the buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = rotary::Interleaved::<f32>::new();
    ///
    /// assert_eq!(buffer.channels(), 0);
    /// buffer.resize_channels(2);
    /// assert_eq!(buffer.channels(), 2);
    /// ```
    pub fn channels(&self) -> usize {
        self.channels
    }

    /// Offset the interleaved buffer and return a wrapped buffer.
    ///
    /// This is provided as a special operation for this buffer kind, because it
    /// can be done more efficiently than what is available through
    /// [Buf::offset].
    pub fn interleaved_offset(&self, offset: usize) -> wrap::Interleaved<&[T]> {
        wrap::interleaved(&self.data[offset * self.channels..], self.channels)
    }

    /// Offset the interleaved buffer and return a mutable wrapped buffer.
    ///
    /// This is provided as a special operation for this buffer kind, because it
    /// can be done more efficiently than what is available through
    /// [Buf::offset].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::{Buf as _, BufMut as _};
    ///
    /// let mut buffer = rotary::Interleaved::with_topology(2, 4);
    ///
    /// buffer.interleaved_offset_mut(2).channel_mut(0).copy_from_slice(&[1.0, 1.0]);
    ///
    /// assert_eq!(buffer.as_slice(), &[0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0])
    /// ```
    pub fn interleaved_offset_mut(&mut self, offset: usize) -> wrap::Interleaved<&mut [T]> {
        wrap::interleaved(&mut self.data[offset * self.channels..], self.channels)
    }

    /// Limit the interleaved buffer and return a wrapped buffer.
    ///
    /// This is provided as a special operation for this buffer kind, because it
    /// can be done more efficiently than what is available through
    /// [Buf::limit].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::{Buf as _, BufMut as _};
    ///
    /// let from = rotary::interleaved![[1.0f32; 4]; 2];
    /// let mut to = rotary::Interleaved::<f32>::with_topology(2, 4);
    ///
    /// to.channel_mut(0).copy_from(from.interleaved_limit(2).channel(0));
    /// assert_eq!(to.as_slice(), &[1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    /// ```
    pub fn interleaved_limit(&self, limit: usize) -> wrap::Interleaved<&[T]> {
        wrap::interleaved(&self.data[..limit * self.channels], self.channels)
    }

    /// Limit the interleaved buffer and return a mutable wrapped buffer.
    ///
    /// This is provided as a special operation for this buffer kind, because it
    /// can be done more efficiently than what is available through
    /// [Buf::limit].
    pub fn interleaved_limit_mut(&mut self, limit: usize) -> wrap::Interleaved<&mut [T]> {
        wrap::interleaved(&mut self.data[..limit * self.channels], self.channels)
    }

    /// Resize to the given number of channels in use.
    ///
    /// If the size of the buffer increases as a result, the new channels will
    /// be zeroed. If the size decreases, the channels that falls outside of the
    /// new size will be dropped.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = rotary::Interleaved::<f32>::new();
    ///
    /// assert_eq!(buffer.channels(), 0);
    /// assert_eq!(buffer.frames(), 0);
    ///
    /// buffer.resize_channels(4);
    /// buffer.resize(256);
    ///
    /// assert_eq!(buffer.channels(), 4);
    /// assert_eq!(buffer.frames(), 256);
    /// ```
    pub fn resize_channels(&mut self, channels: usize) {
        self.inner_resize(channels, self.frames);
    }

    /// Set the size of the buffer. The size is the size of each channel's
    /// buffer.
    ///
    /// If the size of the buffer increases as a result, the new regions in the
    /// frames will be zeroed. If the size decreases, the region will be left
    /// untouched. So if followed by another increase, the data will be "dirty".
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = rotary::Interleaved::<f32>::new();
    ///
    /// assert_eq!(buffer.channels(), 0);
    /// assert_eq!(buffer.frames(), 0);
    ///
    /// buffer.resize_channels(4);
    /// buffer.resize(256);
    ///
    /// assert_eq!(buffer.channels(), 4);
    /// assert_eq!(buffer.frames(), 256);
    ///
    /// {
    ///     let mut chan = buffer.get_mut(1).unwrap();
    ///
    ///     assert_eq!(chan.get(127), Some(0.0));
    ///     *chan.get_mut(127).unwrap() = 42.0;
    ///     assert_eq!(chan.get(127), Some(42.0));
    /// }
    ///
    /// buffer.resize(128);
    /// assert_eq!(buffer.frame(1, 127), Some(42.0));
    ///
    /// buffer.resize(256);
    /// assert_eq!(buffer.frame(1, 127), Some(42.0));
    ///
    /// buffer.resize_channels(2);
    /// assert_eq!(buffer.frame(1, 127), Some(42.0));
    ///
    /// buffer.resize(64);
    /// assert_eq!(buffer.frame(1, 127), None);
    /// ```
    pub fn resize(&mut self, frames: usize) {
        self.inner_resize(self.channels, frames);
    }

    /// Get a reference to a channel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = rotary::Interleaved::<f32>::with_topology(2, 4);
    ///
    /// for (c, s) in buffer.get_mut(0).unwrap().iter_mut().zip(&[1.0, 2.0, 3.0, 4.0]) {
    ///     *c = *s;
    /// }
    ///
    /// for (c, s) in buffer.get_mut(1).unwrap().iter_mut().zip(&[5.0, 6.0, 7.0, 8.0]) {
    ///     *c = *s;
    /// }
    ///
    /// assert_eq!(buffer.get(0).unwrap().iter().nth(2), Some(&3.0));
    /// assert_eq!(buffer.get(1).unwrap().iter().nth(2), Some(&7.0));
    /// ```
    pub fn get(&self, channel: usize) -> Option<Channel<'_, T>> {
        if channel < self.channels {
            Some(Channel {
                inner: RawChannelRef {
                    buffer: self.data.as_ptr(),
                    channel,
                    channels: self.channels,
                    frames: self.frames,
                },
                _marker: marker::PhantomData,
            })
        } else {
            None
        }
    }

    /// Helper to access a single frame in a single channel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = rotary::Interleaved::<f32>::with_topology(2, 256);
    ///
    /// assert_eq!(buffer.frame(1, 128), Some(0.0));
    /// *buffer.frame_mut(1, 128).unwrap() = 1.0;
    /// assert_eq!(buffer.frame(1, 128), Some(1.0));
    /// ```
    pub fn frame(&self, channel: usize, frame: usize) -> Option<T> {
        self.get(channel)?.get(frame)
    }

    /// Get a mutable reference to a channel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = rotary::Interleaved::<f32>::with_topology(2, 4);
    ///
    /// for (c, s) in buffer.get_mut(0).unwrap().iter_mut().zip(&[1.0, 2.0, 3.0, 4.0]) {
    ///     *c = *s;
    /// }
    ///
    /// for (c, s) in buffer.get_mut(1).unwrap().iter_mut().zip(&[5.0, 6.0, 7.0, 8.0]) {
    ///     *c = *s;
    /// }
    ///
    /// assert_eq!(buffer.as_slice(), &[1.0, 5.0, 2.0, 6.0, 3.0, 7.0, 4.0, 8.0]);
    /// ```
    pub fn get_mut(&mut self, channel: usize) -> Option<ChannelMut<'_, T>> {
        if channel < self.channels {
            Some(ChannelMut {
                inner: RawChannelMut {
                    buffer: self.data.as_mut_ptr(),
                    channel,
                    channels: self.channels,
                    frames: self.frames,
                },
                _marker: marker::PhantomData,
            })
        } else {
            None
        }
    }

    /// Helper to access a single frame in a single channel mutably.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = rotary::Interleaved::<f32>::with_topology(2, 256);
    ///
    /// assert_eq!(buffer.frame(1, 128), Some(0.0));
    /// *buffer.frame_mut(1, 128).unwrap() = 1.0;
    /// assert_eq!(buffer.frame(1, 128), Some(1.0));
    /// ```
    pub fn frame_mut(&mut self, channel: usize, frame: usize) -> Option<&mut T> {
        self.get_mut(channel)?.into_mut(frame)
    }

    /// Construct an iterator over all available channels.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = rotary::Interleaved::<f32>::with_topology(2, 4);
    ///
    /// let mut it = buffer.iter_mut();
    ///
    /// for (c, f) in it.next().unwrap().iter_mut().zip(&[1.0, 2.0, 3.0, 4.0]) {
    ///     *c = *f;
    /// }
    ///
    /// for (c, f) in it.next().unwrap().iter_mut().zip(&[5.0, 6.0, 7.0, 8.0]) {
    ///     *c = *f;
    /// }
    ///
    /// let channels = buffer.iter().collect::<Vec<_>>();
    /// let left = channels[0].iter().copied().collect::<Vec<_>>();
    /// let right = channels[1].iter().copied().collect::<Vec<_>>();
    ///
    /// assert_eq!(left, &[1.0, 2.0, 3.0, 4.0]);
    /// assert_eq!(right, &[5.0, 6.0, 7.0, 8.0]);
    /// ```
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            buffer: self.data.as_ptr(),
            channel: 0,
            channels: self.channels,
            frames: self.frames,
            _marker: marker::PhantomData,
        }
    }

    /// Construct a mutable iterator over all available channels.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = rotary::Interleaved::<f32>::with_topology(2, 4);
    ///
    /// let mut it = buffer.iter_mut();
    ///
    /// for (c, f) in it.next().unwrap().iter_mut().zip(&[1.0, 2.0, 3.0, 4.0]) {
    ///     *c = *f;
    /// }
    ///
    /// for (c, f) in it.next().unwrap().iter_mut().zip(&[5.0, 6.0, 7.0, 8.0]) {
    ///     *c = *f;
    /// }
    ///
    /// assert_eq!(buffer.as_slice(), &[1.0, 5.0, 2.0, 6.0, 3.0, 7.0, 4.0, 8.0]);
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            buffer: self.data.as_mut_ptr(),
            channel: 0,
            channels: self.channels,
            frames: self.frames,
            _marker: marker::PhantomData,
        }
    }

    /// The internal resize function for interleaved channel buffers.
    fn inner_resize(&mut self, channels: usize, frames: usize) {
        if self.channels == channels && self.frames == frames {
            return;
        }

        let old_cap = self.data.capacity();
        let new_cap = frames * channels;

        if new_cap > old_cap {
            self.data.reserve(new_cap - old_cap);
            let new_cap = self.data.capacity();

            // Safety: capacity is governed by the underlying vector.
            unsafe {
                ptr::write_bytes(self.data.as_mut_ptr().add(old_cap), 0, new_cap - old_cap);
            }
        }

        if self.channels != channels {
            let len = usize::min(self.channels, channels);

            // Safety: We trust the known lengths lengths.
            unsafe {
                if channels < self.channels {
                    self.inner_shuffle_channels(1..frames, len, channels);
                } else {
                    self.inner_shuffle_channels((1..frames).rev(), len, channels);
                }
            }
        }

        // Safety: since we're decreasing the number of frames we're sure
        // that the data for them is already allocated.
        unsafe {
            self.data.set_len(frames * channels);
        }

        self.channels = channels;
        self.frames = frames;
    }

    /// Internal function to re-shuffle channels.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the ranges of frames, the length and that
    /// the updates `channels` argument is validly within the buffer.
    #[inline]
    unsafe fn inner_shuffle_channels<F>(&mut self, frames: F, len: usize, channels: usize)
    where
        F: IntoIterator<Item = usize>,
    {
        let base = self.data.as_mut_ptr();

        for f in frames {
            let from = f * self.channels;
            let to = f * channels;
            ptr::copy(base.add(from), base.add(to), len)
        }
    }
}

impl<T> fmt::Debug for Interleaved<T>
where
    T: Sample + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T> cmp::PartialEq for Interleaved<T>
where
    T: Sample + cmp::PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<T> cmp::Eq for Interleaved<T> where T: Sample + cmp::Eq {}

impl<T> cmp::PartialOrd for Interleaved<T>
where
    T: Sample + cmp::PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.iter().partial_cmp(other.iter())
    }
}

impl<T> cmp::Ord for Interleaved<T>
where
    T: Sample + cmp::Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.iter().cmp(other.iter())
    }
}

impl<T> hash::Hash for Interleaved<T>
where
    T: Sample + hash::Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        for channel in self.iter() {
            channel.hash(state);
        }
    }
}

impl<T> Buf<T> for Interleaved<T>
where
    T: Sample,
{
    fn frames(&self) -> usize {
        self.frames
    }

    fn channels(&self) -> usize {
        self.channels
    }

    fn channel(&self, channel: usize) -> BufChannel<'_, T> {
        BufChannel::interleaved(&self.data, self.channels, channel)
    }
}

impl<T> BufMut<T> for Interleaved<T>
where
    T: Sample,
{
    fn channel_mut(&mut self, channel: usize) -> BufChannelMut<'_, T> {
        BufChannelMut::interleaved(&mut self.data, self.channels, channel)
    }

    fn resize(&mut self, frames: usize) {
        Self::resize(self, frames);
    }

    fn resize_topology(&mut self, channels: usize, frames: usize) {
        Self::resize(self, frames);
        Self::resize_channels(self, channels);
    }
}

impl<'a, T> IntoIterator for &'a Interleaved<T>
where
    T: Sample,
{
    type IntoIter = Iter<'a, T>;
    type Item = <Self::IntoIter as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Interleaved<T>
where
    T: Sample,
{
    type IntoIter = IterMut<'a, T>;
    type Item = <Self::IntoIter as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// An iterator over the channels in the buffer.
///
/// Created with [Interleaved::iter].
pub struct Iter<'a, T>
where
    T: Sample,
{
    buffer: *const T,
    channel: usize,
    channels: usize,
    frames: usize,
    _marker: marker::PhantomData<&'a T>,
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
/// Created with [Interleaved::iter_mut].
pub struct IterMut<'a, T>
where
    T: Sample,
{
    buffer: *mut T,
    channel: usize,
    channels: usize,
    frames: usize,
    _marker: marker::PhantomData<&'a mut T>,
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
