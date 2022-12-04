use core::cmp;
use core::fmt;
use core::hash;
use core::ops;
use core::ptr;

use audio_core::{Buf, BufMut, ExactSizeBuf, ResizableBuf, Sample, UniformBuf};

use crate::buf::sequential::{Iter, IterMut};
use crate::channel::{LinearChannel, LinearChannelMut};
use crate::frame::{RawSequential, SequentialFrame, SequentialFramesIter};

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
/// ```
/// let mut buf = audio::buf::Sequential::<f32>::with_topology(2, 4);
/// buf[0].copy_from_slice(&[1.0, 2.0, 3.0, 4.0]);
/// buf[1].copy_from_slice(&[2.0, 3.0, 4.0, 5.0]);
///
/// buf.resize(3);
///
/// assert_eq!(&buf[0], &[1.0, 2.0, 3.0]);
/// assert_eq!(&buf[1], &[2.0, 3.0, 4.0]);
///
/// buf.resize(4);
///
/// assert_eq!(&buf[0], &[1.0, 2.0, 3.0, 2.0]); // <- 2.0 is stale data.
/// assert_eq!(&buf[1], &[2.0, 3.0, 4.0, 5.0]); // <- 5.0 is stale data.
/// ```
///
/// To access the full, currently assumed *valid* slice you can use
/// [Sequential::as_slice] or [Sequential::into_vec].
///
/// ```
/// let mut buf = audio::buf::Sequential::<f32>::with_topology(2, 4);
/// buf[0].copy_from_slice(&[1.0, 2.0, 3.0, 4.0]);
/// buf[1].copy_from_slice(&[2.0, 3.0, 4.0, 5.0]);
///
/// buf.resize(3);
///
/// assert_eq!(buf.as_slice(), &[1.0, 2.0, 3.0, 2.0, 3.0, 4.0]);
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
    /// ```
    /// let buf = audio::buf::Sequential::<f32>::new();
    ///
    /// assert_eq!(buf.frames(), 0);
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
    /// ```
    /// let mut buf = audio::buf::Sequential::<f32>::with_topology(4, 256);
    ///
    /// assert_eq!(buf.frames(), 256);
    /// assert_eq!(buf.channels(), 4);
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
    /// ```
    /// let buf = audio::sequential![[2.0; 256]; 4];
    ///
    /// assert_eq!(buf.frames(), 256);
    /// assert_eq!(buf.channels(), 4);
    ///
    /// for chan in &buf {
    ///     assert_eq!(chan.as_ref(), vec![2.0; 256]);
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
    /// ```
    /// let buf = audio::buf::Sequential::from_frames([1.0, 2.0, 3.0, 4.0], 2);
    ///
    /// assert_eq!(buf.frames(), 4);
    /// assert_eq!(buf.channels(), 2);
    ///
    /// assert_eq!(buf.as_slice(), &[1.0, 2.0, 3.0, 4.0, 1.0, 2.0, 3.0, 4.0]);
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
                data.extend(frames);
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
    /// ```
    /// let buf = audio::buf::Sequential::from_array([[1; 4]; 2]);
    ///
    /// assert_eq!(buf.frames(), 4);
    /// assert_eq!(buf.channels(), 2);
    ///
    /// assert_eq! {
    ///     buf.as_slice(),
    ///     &[1, 1, 1, 1, 1, 1, 1, 1],
    /// }
    /// ```
    ///
    /// Using a specific array topology.
    ///
    /// ```
    /// let buf = audio::buf::Sequential::from_array([[1, 2, 3, 4], [5, 6, 7, 8]]);
    ///
    /// assert_eq!(buf.frames(), 4);
    /// assert_eq!(buf.channels(), 2);
    ///
    /// assert_eq! {
    ///     buf.as_slice(),
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

            for frames in channels {
                for f in frames {
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
    /// ```
    /// let mut buf = audio::buf::Sequential::<f32>::with_topology(2, 4);
    /// buf[0].copy_from_slice(&[1.0, 2.0, 3.0, 4.0]);
    /// buf[1].copy_from_slice(&[2.0, 3.0, 4.0, 5.0]);
    ///
    /// buf.resize(3);
    ///
    /// assert_eq!(buf.into_vec(), vec![1.0, 2.0, 3.0, 2.0, 3.0, 4.0])
    /// ```
    pub fn into_vec(self) -> Vec<T> {
        self.data
    }

    /// Access the underlying vector as a slice.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut buf = audio::buf::Sequential::<f32>::with_topology(2, 4);
    ///
    /// buf[0].copy_from_slice(&[1.0, 2.0, 3.0, 4.0]);
    /// buf[1].copy_from_slice(&[2.0, 3.0, 4.0, 5.0]);
    ///
    /// buf.resize(3);
    ///
    /// assert_eq!(buf.as_slice(), &[1.0, 2.0, 3.0, 2.0, 3.0, 4.0])
    /// ```
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    /// Access the underlying vector as a mutable slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use audio::{Buf, Channel};
    ///
    /// let mut buf = audio::buf::Sequential::<u32>::with_topology(2, 4);
    /// buf.as_slice_mut().copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
    ///
    /// assert_eq! {
    ///     buf.get(0).unwrap(),
    ///     [1u32, 2, 3, 4],
    /// };
    ///
    /// assert_eq! {
    ///     buf.get(1).unwrap(),
    ///     [5u32, 6, 7, 8],
    /// };
    ///
    /// assert_eq!(buf.as_slice(), &[1, 2, 3, 4, 5, 6, 7, 8]);
    /// ```
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        &mut self.data
    }

    /// Get the capacity of the buffer in number of frames.
    ///
    /// The underlying buffer over-allocates a bit, so this will report the
    /// exact capacity available in the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut buf = audio::buf::Sequential::<f32>::new();
    ///
    /// assert_eq!(buf.capacity(), 0);
    ///
    /// buf.resize(11);
    /// assert_eq!(buf.capacity(), 0);
    ///
    /// buf.resize_channels(2);
    /// assert_eq!(buf.capacity(), 22);
    ///
    /// buf.resize(12);
    /// assert_eq!(buf.capacity(), 44);
    ///
    /// buf.resize(24);
    /// assert_eq!(buf.capacity(), 44);
    /// ```
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Get how many frames there are in the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut buf = audio::buf::Sequential::<f32>::new();
    ///
    /// assert_eq!(buf.frames(), 0);
    /// buf.resize(256);
    /// assert_eq!(buf.frames(), 256);
    /// ```
    pub fn frames(&self) -> usize {
        self.frames
    }

    /// Get how many channels there are in the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut buf = audio::buf::Sequential::<f32>::new();
    ///
    /// assert_eq!(buf.channels(), 0);
    /// buf.resize_channels(2);
    /// assert_eq!(buf.channels(), 2);
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
    /// let buf = audio::buf::Sequential::<f32>::with_topology(4, 256);
    ///
    /// let all_zeros = vec![0.0; 256];
    ///
    /// for chan in buf.iter() {
    ///     assert_eq!(chan.as_ref(), &all_zeros[..]);
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
    /// let mut buf = audio::buf::Sequential::<f32>::with_topology(4, 256);
    /// let mut rng = rand::thread_rng();
    ///
    /// for mut chan in buf.iter_mut() {
    ///     rng.fill(chan.as_mut());
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
    /// ```
    /// let mut buf = audio::buf::Sequential::<f32>::new();
    ///
    /// assert_eq!(buf.channels(), 0);
    /// assert_eq!(buf.frames(), 0);
    ///
    /// buf.resize_channels(4);
    /// buf.resize(256);
    ///
    /// assert_eq!(buf.channels(), 4);
    /// assert_eq!(buf.frames(), 256);
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
    /// ```
    /// let mut buf = audio::buf::Sequential::<f32>::new();
    ///
    /// assert_eq!(buf.channels(), 0);
    /// assert_eq!(buf.frames(), 0);
    ///
    /// buf.resize_channels(4);
    /// buf.resize(256);
    ///
    /// assert_eq!(buf[1][128], 0.0);
    /// buf[1][128] = 42.0;
    ///
    /// assert_eq!(buf.channels(), 4);
    /// assert_eq!(buf.frames(), 256);
    /// ```
    ///
    /// Decreasing and increasing the size will modify the underlying buffer:
    ///
    /// ```
    /// # let mut buf = audio::buf::Sequential::<f32>::with_topology(4, 256);
    /// assert_eq!(buf[1][128], 0.0);
    /// buf[1][128] = 42.0;
    ///
    /// buf.resize(64);
    /// assert!(buf[1].get(128).is_none());
    ///
    /// buf.resize(256);
    /// assert_eq!(buf[1][128], 0.0);
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
    /// ```
    /// let mut buf = audio::buf::Sequential::<f32>::new();
    ///
    /// buf.resize_channels(4);
    /// buf.resize(128);
    ///
    /// let expected = (0..128).map(|v| v as f32).collect::<Vec<_>>();
    ///
    /// for mut chan in buf.iter_mut() {
    ///     for (s, v) in chan.iter_mut().zip(&expected) {
    ///         *s = *v;
    ///     }
    /// }
    ///
    /// assert_eq!(buf.get(0).unwrap(), &expected[..]);
    /// assert_eq!(buf.get(1).unwrap(), &expected[..]);
    /// assert_eq!(buf.get(2).unwrap(), &expected[..]);
    /// assert_eq!(buf.get(3).unwrap(), &expected[..]);
    /// assert!(buf.get(4).is_none());
    ///
    /// buf.resize_channels(2);
    ///
    /// assert_eq!(buf.get(0).unwrap(), &expected[..]);
    /// assert_eq!(buf.get(1).unwrap(), &expected[..]);
    /// assert!(buf.get(2).is_none());
    ///
    /// // shrink
    /// buf.resize(64);
    ///
    /// assert_eq!(buf.get(0).unwrap(), &expected[..64]);
    /// assert_eq!(buf.get(1).unwrap(), &expected[..64]);
    /// assert!(buf.get(2).is_none());
    ///
    /// // increase - this causes some weirdness.
    /// buf.resize(128);
    ///
    /// let first_overlapping = expected[..64]
    ///     .iter()
    ///     .chain(expected[..64].iter())
    ///     .copied()
    ///     .collect::<Vec<_>>();
    ///
    /// assert_eq!(buf.get(0).unwrap(), &first_overlapping[..]);
    /// // Note: second channel matches perfectly up with an old channel that was
    /// // masked out.
    /// assert_eq!(buf.get(1).unwrap(), &expected[..]);
    /// assert!(buf.get(2).is_none());
    /// ```
    pub fn resize(&mut self, frames: usize)
    where
        T: Sample,
    {
        self.resize_inner(self.channels, self.frames, self.channels, frames);
    }

    /// Get a reference to the buffer of the given channel.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut buf = audio::buf::Sequential::<f32>::new();
    ///
    /// buf.resize_channels(4);
    /// buf.resize(256);
    ///
    /// let expected = vec![0.0; 256];
    ///
    /// assert_eq!(buf.get(0).unwrap(), &expected[..]);
    /// assert_eq!(buf.get(1).unwrap(), &expected[..]);
    /// assert_eq!(buf.get(2).unwrap(), &expected[..]);
    /// assert_eq!(buf.get(3).unwrap(), &expected[..]);
    /// assert!(buf.get(4).is_none());
    /// ```
    pub fn get(&self, channel: usize) -> Option<LinearChannel<'_, T>> {
        if channel >= self.channels {
            return None;
        }

        let data = self.data.get(channel * self.frames..)?.get(..self.frames)?;
        Some(LinearChannel::new(data))
    }

    /// Get a mutable reference to the buffer of the given channel.
    ///
    /// # Examples
    ///
    /// ```
    /// use rand::Rng as _;
    ///
    /// let mut buf = audio::buf::Sequential::<f32>::new();
    ///
    /// buf.resize_channels(2);
    /// buf.resize(256);
    ///
    /// let mut rng = rand::thread_rng();
    ///
    /// if let Some(mut left) = buf.get_mut(0) {
    ///     rng.fill(left.as_mut());
    /// }
    ///
    /// if let Some(mut right) = buf.get_mut(1) {
    ///     rng.fill(right.as_mut());
    /// }
    /// ```
    pub fn get_mut(&mut self, channel: usize) -> Option<LinearChannelMut<'_, T>> {
        if channel >= self.channels {
            return None;
        }

        let data = self
            .data
            .get_mut(channel * self.frames..)?
            .get_mut(..self.frames)?;

        Some(LinearChannelMut::new(data))
    }

    /// Reserve the given capacity in this buffer ensuring it can take at least
    /// `capacity` elements in total before needing to re-allocate again.
    pub fn reserve(&mut self, capacity: usize) {
        let old_cap = self.data.capacity();

        if old_cap < capacity {
            self.data.reserve(capacity - old_cap);
        }
    }

    /// Access the raw sequential buffer.
    fn as_raw(&self) -> RawSequential<T> {
        // SAFETY: construction of the current buffer ensures this is safe.
        unsafe { RawSequential::new(&self.data, self.channels, self.frames) }
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
            Some(slice) => slice.into_ref(),
            None => panic!("index `{}` is not a channel", index),
        }
    }
}

impl<T> ops::IndexMut<usize> for Sequential<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match self.get_mut(index) {
            Some(slice) => slice.into_mut(),
            None => panic!("index `{}` is not a channel", index,),
        }
    }
}

impl<T> ExactSizeBuf for Sequential<T>
where
    T: Copy,
{
    fn frames(&self) -> usize {
        self.frames
    }
}

impl<T> Buf for Sequential<T>
where
    T: Copy,
{
    type Sample = T;

    type Channel<'this> = LinearChannel<'this, Self::Sample>
    where
        Self::Sample: 'this;

    type Iter<'this> = Iter<'this, T>
    where
        Self: 'this;

    fn frames_hint(&self) -> Option<usize> {
        Some(self.frames)
    }

    fn channels(&self) -> usize {
        (*self).channels()
    }

    fn get(&self, channel: usize) -> Option<Self::Channel<'_>> {
        (*self).get(channel)
    }

    fn iter(&self) -> Self::Iter<'_> {
        (*self).iter()
    }
}

impl<T> UniformBuf for Sequential<T>
where
    T: Copy,
{
    type Frame<'this> = SequentialFrame<'this, T>
    where
        Self: 'this;

    type FramesIter<'this> = SequentialFramesIter<'this, T>
    where
        Self: 'this;

    #[inline]
    fn get_frame(&self, frame: usize) -> Option<Self::Frame<'_>> {
        if frame >= self.frames {
            return None;
        }

        Some(SequentialFrame::new(frame, self.as_raw()))
    }

    #[inline]
    fn iter_frames(&self) -> Self::FramesIter<'_> {
        SequentialFramesIter::new(0, self.as_raw())
    }
}

impl<T> ResizableBuf for Sequential<T>
where
    T: Sample,
{
    fn try_reserve(&mut self, capacity: usize) -> bool {
        self.reserve(capacity);
        true
    }

    fn resize(&mut self, frames: usize) {
        Self::resize(self, frames);
    }

    fn resize_topology(&mut self, channels: usize, frames: usize) {
        Self::resize(self, frames);
        Self::resize_channels(self, channels);
    }
}

impl<T> BufMut for Sequential<T>
where
    T: Copy,
{
    type ChannelMut<'this> = LinearChannelMut<'this, Self::Sample>
    where
        Self: 'this;

    type IterMut<'this> = IterMut<'this, T>
    where
        Self: 'this;

    fn get_mut(&mut self, channel: usize) -> Option<Self::ChannelMut<'_>> {
        (*self).get_mut(channel)
    }

    fn copy_channel(&mut self, from: usize, to: usize) {
        // Safety: We're calling the copy function with internal parameters
        // which are guaranteed to be correct.
        unsafe {
            crate::utils::copy_channels_sequential(
                ptr::NonNull::new_unchecked(self.data.as_mut_ptr()),
                self.channels,
                self.frames,
                from,
                to,
            )
        }
    }

    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        (*self).iter_mut()
    }
}
