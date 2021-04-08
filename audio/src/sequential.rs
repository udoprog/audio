//! A dynamically sized, multi-channel sequential audio buffer.

use audio_core::{
    Buf, Channel, ChannelMut, Channels, ChannelsMut, ExactSizeBuf, ResizableBuf, Sample,
};
use std::cmp;
use std::fmt;
use std::hash;
use std::ops;
use std::ptr;

mod iter;
pub use self::iter::{Iter, IterMut};

/// A dynamically sized, multi-channel sequential audio buffer.
///
/// A *sequential* audio buffer stores all audio data sequentially in memory,
/// one channel after another.
///
/// An audio buffer can only be resized if it contains a type which is
/// sample-apt For more information of what this means, see [Sample].
///
/// Resizing the buffer might therefore cause a fair bit of copying, and for the
/// worst cases, this might result in having to copy a memory region
/// byte-by-byte since they might overlap.
///
/// Resized regions also aren't zeroed, so certain operations might cause stale
/// data to be visible after a resize.
///
/// ```rust
/// let mut buffer = audio::Sequential::<f32>::with_topology(2, 4);
/// buffer[0].copy_from_slice(&[1.0, 2.0, 3.0, 4.0]);
/// buffer[1].copy_from_slice(&[2.0, 3.0, 4.0, 5.0]);
///
/// buffer.resize(3);
///
/// assert_eq!(&buffer[0], &[1.0, 2.0, 3.0]);
/// assert_eq!(&buffer[1], &[2.0, 3.0, 4.0]);
///
/// buffer.resize(4);
///
/// assert_eq!(&buffer[0], &[1.0, 2.0, 3.0, 2.0]); // <- 2.0 is stale data.
/// assert_eq!(&buffer[1], &[2.0, 3.0, 4.0, 5.0]); // <- 5.0 is stale data.
/// ```
///
/// To access the full, currently assumed *valid* slice you can use
/// [Sequential::as_slice] or [Sequential::into_vec].
///
/// ```rust
/// let mut buffer = audio::Sequential::<f32>::with_topology(2, 4);
/// buffer[0].copy_from_slice(&[1.0, 2.0, 3.0, 4.0]);
/// buffer[1].copy_from_slice(&[2.0, 3.0, 4.0, 5.0]);
///
/// buffer.resize(3);
///
/// assert_eq!(buffer.as_slice(), &[1.0, 2.0, 3.0, 2.0, 3.0, 4.0]);
/// ```
#[derive(Default)]
pub struct Sequential<T> {
    data: Vec<T>,
    channels: usize,
    frames: usize,
}

impl<T> Sequential<T> {
    /// Construct a new empty audio buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = audio::Sequential::<f32>::new();
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
    /// let mut buffer = audio::Sequential::<f32>::with_topology(4, 256);
    ///
    /// assert_eq!(buffer.frames(), 256);
    /// assert_eq!(buffer.channels(), 4);
    /// ```
    pub fn with_topology(channels: usize, frames: usize) -> Self
    where
        T: Sample,
    {
        Self {
            data: vec![T::ZERO; channels * frames],
            channels,
            frames,
        }
    }

    /// Allocate an audio buffer from a fixed-size array.
    ///
    /// See [sequential!].
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = audio::sequential![[2.0; 256]; 4];
    ///
    /// assert_eq!(buffer.frames(), 256);
    /// assert_eq!(buffer.channels(), 4);
    ///
    /// for chan in &buffer {
    ///     assert_eq!(chan, vec![2.0; 256]);
    /// }
    /// ```
    pub fn from_vec(data: Vec<T>, channels: usize, frames: usize) -> Self {
        Self {
            data,
            channels,
            frames,
        }
    }

    /// Allocate an audio buffer from a fixed-size array acting as a template
    /// for all the channels.
    ///
    /// See [sequential!].
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = audio::Sequential::from_frames([1.0, 2.0, 3.0, 4.0], 2);
    ///
    /// assert_eq!(buffer.frames(), 4);
    /// assert_eq!(buffer.channels(), 2);
    ///
    /// assert_eq!(buffer.as_slice(), &[1.0, 2.0, 3.0, 4.0, 1.0, 2.0, 3.0, 4.0]);
    /// ```
    pub fn from_frames<const N: usize>(frames: [T; N], channels: usize) -> Self
    where
        T: Copy,
    {
        return Self {
            data: data_from_frames(frames, channels),
            channels,
            frames: N,
        };

        fn data_from_frames<T, const N: usize>(frames: [T; N], channels: usize) -> Vec<T>
        where
            T: Copy,
        {
            let mut data = Vec::with_capacity(N * channels);

            for _ in 0..channels {
                data.extend(std::array::IntoIter::new(frames));
            }

            data
        }
    }

    /// Allocate a sequential audio buffer from a fixed-size array.
    ///
    /// See [sequential!].
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = audio::Sequential::from_array([[1; 4]; 2]);
    ///
    /// assert_eq!(buffer.frames(), 4);
    /// assert_eq!(buffer.channels(), 2);
    ///
    /// assert_eq! {
    ///     buffer.as_slice(),
    ///     &[1, 1, 1, 1, 1, 1, 1, 1],
    /// }
    /// ```
    ///
    /// Using a specific array topology.
    ///
    /// ```rust
    /// let mut buffer = audio::Sequential::from_array([[1, 2, 3, 4], [5, 6, 7, 8]]);
    ///
    /// assert_eq!(buffer.frames(), 4);
    /// assert_eq!(buffer.channels(), 2);
    ///
    /// assert_eq! {
    ///     buffer.as_slice(),
    ///     &[1, 2, 3, 4, 5, 6, 7, 8],
    /// }
    /// ```
    pub fn from_array<const F: usize, const C: usize>(channels: [[T; F]; C]) -> Self
    where
        T: Copy,
    {
        return Self {
            data: data_from_array(channels),
            channels: C,
            frames: F,
        };

        #[inline]
        fn data_from_array<T, const F: usize, const C: usize>(channels: [[T; F]; C]) -> Vec<T> {
            let mut data = Vec::with_capacity(C * F);

            for frames in std::array::IntoIter::new(channels) {
                for f in std::array::IntoIter::new(frames) {
                    data.push(f);
                }
            }

            data
        }
    }

    /// Take ownership of the backing vector.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = audio::Sequential::<f32>::with_topology(2, 4);
    /// buffer[0].copy_from_slice(&[1.0, 2.0, 3.0, 4.0]);
    /// buffer[1].copy_from_slice(&[2.0, 3.0, 4.0, 5.0]);
    ///
    /// buffer.resize(3);
    ///
    /// assert_eq!(buffer.into_vec(), vec![1.0, 2.0, 3.0, 2.0, 3.0, 4.0])
    /// ```
    pub fn into_vec(self) -> Vec<T> {
        self.data
    }

    /// Access the underlying vector as a slice.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = audio::Sequential::<f32>::with_topology(2, 4);
    ///
    /// buffer[0].copy_from_slice(&[1.0, 2.0, 3.0, 4.0]);
    /// buffer[1].copy_from_slice(&[2.0, 3.0, 4.0, 5.0]);
    ///
    /// buffer.resize(3);
    ///
    /// assert_eq!(buffer.as_slice(), &[1.0, 2.0, 3.0, 2.0, 3.0, 4.0])
    /// ```
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    /// Get the number of frames in the channels of an audio buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = audio::Sequential::<f32>::new();
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
    /// let mut buffer = audio::Sequential::<f32>::new();
    ///
    /// assert_eq!(buffer.channels(), 0);
    /// buffer.resize_channels(2);
    /// assert_eq!(buffer.channels(), 2);
    /// ```
    pub fn channels(&self) -> usize {
        self.channels
    }

    /// Construct an iterator over all available channels.
    ///
    /// # Examples
    ///
    /// ```
    /// use rand::Rng as _;
    ///
    /// let mut buffer = audio::Sequential::<f32>::with_topology(4, 256);
    ///
    /// let all_zeros = vec![0.0; 256];
    ///
    /// for chan in buffer.iter() {
    ///     assert_eq!(chan, &all_zeros[..]);
    /// }
    /// ```
    pub fn iter(&self) -> Iter<'_, T> {
        Iter::new(&self.data, self.frames)
    }

    /// Construct a mutable iterator over all available channels.
    ///
    /// # Examples
    ///
    /// ```
    /// use rand::Rng as _;
    ///
    /// let mut buffer = audio::Sequential::<f32>::with_topology(4, 256);
    /// let mut rng = rand::thread_rng();
    ///
    /// for chan in buffer.iter_mut() {
    ///     rng.fill(chan);
    /// }
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut::new(&mut self.data, self.frames)
    }

    /// Set the number of channels in use.
    ///
    /// If the size of the buffer increases as a result, the new channels will
    /// be zeroed. If the size decreases, the channels that falls outside of the
    /// new size will be dropped.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = audio::Sequential::<f32>::new();
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
    pub fn resize_channels(&mut self, channels: usize)
    where
        T: Sample,
    {
        self.resize_inner(self.channels, self.frames, channels, self.frames);
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
    /// let mut buffer = audio::Sequential::<f32>::new();
    ///
    /// assert_eq!(buffer.channels(), 0);
    /// assert_eq!(buffer.frames(), 0);
    ///
    /// buffer.resize_channels(4);
    /// buffer.resize(256);
    ///
    /// assert_eq!(buffer[1][128], 0.0);
    /// buffer[1][128] = 42.0;
    ///
    /// assert_eq!(buffer.channels(), 4);
    /// assert_eq!(buffer.frames(), 256);
    /// ```
    ///
    /// Decreasing and increasing the size will modify the underlying buffer:
    ///
    /// ```rust
    /// # let mut buffer = audio::Sequential::<f32>::with_topology(4, 256);
    /// assert_eq!(buffer[1][128], 0.0);
    /// buffer[1][128] = 42.0;
    ///
    /// buffer.resize(64);
    /// assert!(buffer[1].get(128).is_none());
    ///
    /// buffer.resize(256);
    /// assert_eq!(buffer[1][128], 0.0);
    /// ```
    ///
    /// # Stale data
    ///
    /// Resizing a channel doesn't "free" the underlying data or zero previously
    /// initialized regions.
    ///
    /// Old regions which were previously sized out and ignored might contain
    /// stale data from previous uses. So this should be kept in mind when
    /// resizing this buffer dynamically.
    ///
    /// ```rust
    /// let mut buffer = audio::Sequential::<f32>::new();
    ///
    /// buffer.resize_channels(4);
    /// buffer.resize(128);
    ///
    /// let expected = (0..128).map(|v| v as f32).collect::<Vec<_>>();
    ///
    /// for chan in buffer.iter_mut() {
    ///     for (s, v) in chan.iter_mut().zip(&expected) {
    ///         *s = *v;
    ///     }
    /// }
    ///
    /// assert_eq!(buffer.get(0), Some(&expected[..]));
    /// assert_eq!(buffer.get(1), Some(&expected[..]));
    /// assert_eq!(buffer.get(2), Some(&expected[..]));
    /// assert_eq!(buffer.get(3), Some(&expected[..]));
    /// assert_eq!(buffer.get(4), None);
    ///
    /// buffer.resize_channels(2);
    ///
    /// assert_eq!(buffer.get(0), Some(&expected[..]));
    /// assert_eq!(buffer.get(1), Some(&expected[..]));
    /// assert_eq!(buffer.get(2), None);
    ///
    /// // shrink
    /// buffer.resize(64);
    ///
    /// assert_eq!(buffer.get(0), Some(&expected[..64]));
    /// assert_eq!(buffer.get(1), Some(&expected[..64]));
    /// assert_eq!(buffer.get(2), None);
    ///
    /// // increase - this causes some weirdness.
    /// buffer.resize(128);
    ///
    /// let first_overlapping = expected[..64]
    ///     .iter()
    ///     .chain(expected[..64].iter())
    ///     .copied()
    ///     .collect::<Vec<_>>();
    ///
    /// assert_eq!(buffer.get(0), Some(&first_overlapping[..]));
    /// // Note: second channel matches perfectly up with an old channel that was
    /// // masked out.
    /// assert_eq!(buffer.get(1), Some(&expected[..]));
    /// assert_eq!(buffer.get(2), None);
    /// ```
    pub fn resize(&mut self, frames: usize)
    where
        T: Sample,
    {
        self.resize_inner(self.channels, self.frames, self.channels, frames);
    }

    /// Get the capacity of the interleaved buffer in number of frames.
    ///
    /// The underlying buffer over-allocates a bit, so this will report the
    /// exact capacity available in the interleaved buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = audio::Sequential::<f32>::new();
    ///
    /// assert_eq!(buffer.capacity(), 0);
    ///
    /// buffer.resize(11);
    /// assert_eq!(buffer.capacity(), 0);
    ///
    /// buffer.resize_channels(2);
    /// assert_eq!(buffer.capacity(), 11);
    ///
    /// buffer.resize(12);
    /// assert_eq!(buffer.capacity(), 22);
    ///
    /// buffer.resize(22);
    /// assert_eq!(buffer.capacity(), 22);
    /// ```
    pub fn capacity(&self) -> usize {
        if self.channels == 0 {
            0
        } else {
            self.data.capacity() / self.channels
        }
    }

    /// Get a reference to the buffer of the given channel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = audio::Sequential::<f32>::new();
    ///
    /// buffer.resize_channels(4);
    /// buffer.resize(256);
    ///
    /// let expected = vec![0.0; 256];
    ///
    /// assert_eq!(Some(&expected[..]), buffer.get(0));
    /// assert_eq!(Some(&expected[..]), buffer.get(1));
    /// assert_eq!(Some(&expected[..]), buffer.get(2));
    /// assert_eq!(Some(&expected[..]), buffer.get(3));
    /// assert_eq!(None, buffer.get(4));
    /// ```
    pub fn get(&self, channel: usize) -> Option<&[T]> {
        if channel >= self.channels {
            return None;
        }

        self.data.get(channel * self.frames..)?.get(..self.frames)
    }

    /// Get a mutable reference to the buffer of the given channel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rand::Rng as _;
    ///
    /// let mut buffer = audio::Sequential::<f32>::new();
    ///
    /// buffer.resize_channels(2);
    /// buffer.resize(256);
    ///
    /// let mut rng = rand::thread_rng();
    ///
    /// if let Some(left) = buffer.get_mut(0) {
    ///     rng.fill(left);
    /// }
    ///
    /// if let Some(right) = buffer.get_mut(1) {
    ///     rng.fill(right);
    /// }
    /// ```
    pub fn get_mut(&mut self, channel: usize) -> Option<&mut [T]> {
        if channel >= self.channels {
            return None;
        }

        self.data
            .get_mut(channel * self.frames..)?
            .get_mut(..self.frames)
    }

    fn resize_inner(
        &mut self,
        from_channels: usize,
        from_frames: usize,
        to_channels: usize,
        to_frames: usize,
    ) where
        T: Sample,
    {
        if to_channels == 0 || to_frames == 0 {
            self.channels = to_channels;
            self.frames = to_frames;
            return;
        } else if self.channels == to_channels && self.frames == to_frames {
            return;
        }

        let old_cap = self.data.capacity();
        let new_len = to_channels * to_frames;

        if old_cap < new_len {
            let additional = new_len - self.data.capacity();
            self.data.reserve(additional);

            // zero the additional capacity.
            unsafe {
                ptr::write_bytes(
                    self.data.as_mut_ptr().add(old_cap),
                    0,
                    self.data.capacity() - old_cap,
                );
            }
        }

        if from_frames < to_frames {
            for chan in (0..from_channels).rev() {
                unsafe {
                    let src = self.data.as_mut_ptr().add(chan * from_frames);
                    let dst = self.data.as_mut_ptr().add(chan * to_frames);
                    ptr::copy(src, dst, from_frames);
                }
            }
        } else {
            for chan in 0..from_channels {
                unsafe {
                    let src = self.data.as_mut_ptr().add(chan * from_frames);
                    let dst = self.data.as_mut_ptr().add(chan * to_frames);
                    ptr::copy(src, dst, from_frames);
                }
            }
        }

        // Resize underlying storage.
        unsafe {
            self.data.set_len(new_len);
        }

        self.channels = to_channels;
        self.frames = to_frames;
    }
}

impl<T> fmt::Debug for Sequential<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T> cmp::PartialEq for Sequential<T>
where
    T: cmp::PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<T> cmp::Eq for Sequential<T> where T: cmp::Eq {}

impl<T> cmp::PartialOrd for Sequential<T>
where
    T: cmp::PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.iter().partial_cmp(other.iter())
    }
}

impl<T> cmp::Ord for Sequential<T>
where
    T: cmp::Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.iter().cmp(other.iter())
    }
}

impl<T> hash::Hash for Sequential<T>
where
    T: hash::Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        for channel in self.iter() {
            channel.hash(state);
        }
    }
}

impl<'a, T> IntoIterator for &'a Sequential<T> {
    type IntoIter = Iter<'a, T>;
    type Item = <Self::IntoIter as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Sequential<T> {
    type IntoIter = IterMut<'a, T>;
    type Item = <Self::IntoIter as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> ops::Index<usize> for Sequential<T> {
    type Output = [T];

    fn index(&self, index: usize) -> &Self::Output {
        match self.get(index) {
            Some(slice) => slice,
            None => panic!("index `{}` is not a channel", index),
        }
    }
}

impl<T> ops::IndexMut<usize> for Sequential<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match self.get_mut(index) {
            Some(slice) => slice,
            None => panic!("index `{}` is not a channel", index,),
        }
    }
}

impl<T> ExactSizeBuf for Sequential<T> {
    fn frames(&self) -> usize {
        self.frames
    }
}

impl<T> Buf for Sequential<T> {
    fn frames_hint(&self) -> Option<usize> {
        Some(self.frames)
    }

    fn channels(&self) -> usize {
        self.channels
    }
}

impl<T> Channels<T> for Sequential<T> {
    fn channel(&self, channel: usize) -> Channel<'_, T> {
        let data = &self.data[self.frames * channel..];
        Channel::linear(&data[..self.frames])
    }
}

impl<T> ResizableBuf for Sequential<T>
where
    T: Sample,
{
    fn resize(&mut self, frames: usize) {
        Self::resize(self, frames);
    }

    fn resize_topology(&mut self, channels: usize, frames: usize) {
        Self::resize(self, frames);
        Self::resize_channels(self, channels);
    }
}

impl<T> ChannelsMut<T> for Sequential<T>
where
    T: Copy,
{
    fn channel_mut(&mut self, channel: usize) -> ChannelMut<'_, T> {
        let data = &mut self.data[self.frames * channel..];
        ChannelMut::linear(&mut data[..self.frames])
    }

    fn copy_channels(&mut self, from: usize, to: usize) {
        // Safety: We're calling the copy function with internal parameters
        // which are guaranteed to be correct.
        unsafe {
            crate::utils::copy_channels_sequential(
                self.data.as_mut_ptr(),
                self.channels,
                self.frames,
                from,
                to,
            )
        }
    }
}
