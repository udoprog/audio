//! A dynamically sized, multi-channel interleaved audio buffer.

use crate::channel::{InterleavedMut, InterleavedRef};
use core::{Buf, BufMut, ExactSizeBuf, InterleavedBuf, InterleavedBufMut, ResizableBuf, Sample};
use std::cmp;
use std::fmt;
use std::hash;
use std::ptr;

mod iter;
pub use self::iter::{Iter, IterMut};

/// A dynamically sized, multi-channel interleaved audio buffer.
///
/// An audio buffer can only be resized if it contains a type which is
/// sample-apt For more information of what this means, see [Sample].
///
/// An *interleaved* audio buffer stores all audio data interleaved in memory,
/// one sample from each channel in sequence until we're out of samples. This
/// naturally makes the buffer a bit harder to work with, and we have to rely on
/// iterators to access logical channels.
///
/// Resized regions aren't zeroed, so certain operations might cause stale data
/// to be visible after a resize.
///
/// ```
/// let mut buf = audio::buf::Interleaved::<f32>::with_topology(2, 4);
///
/// for (c, s) in buf
///     .get_mut(0)
///     .unwrap()
///     .iter_mut()
///     .zip(&[1.0, 2.0, 3.0, 4.0])
/// {
///     *c = *s;
/// }
///
/// for (c, s) in buf
///     .get_mut(1)
///     .unwrap()
///     .iter_mut()
///     .zip(&[5.0, 6.0, 7.0, 8.0])
/// {
///     *c = *s;
/// }
///
/// assert_eq!(buf.as_slice(), &[1.0, 5.0, 2.0, 6.0, 3.0, 7.0, 4.0, 8.0]);
/// ```
#[derive(Default)]
pub struct Interleaved<T> {
    data: Vec<T>,
    channels: usize,
    frames: usize,
}

impl<T> Interleaved<T> {
    /// Construct a new empty audio buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// let buf = audio::buf::Interleaved::<f32>::new();
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
    /// let buf = audio::buf::Interleaved::<f32>::with_topology(4, 256);
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
    /// See [dynamic!].
    ///
    /// # Examples
    ///
    /// ```
    /// let buf = audio::interleaved![[2.0; 256]; 4];
    ///
    /// assert_eq!(buf.frames(), 256);
    /// assert_eq!(buf.channels(), 4);
    ///
    /// for chan in &buf {
    ///     assert!(chan.iter().eq([2.0; 256]));
    /// }
    /// ```
    pub fn from_vec(data: Vec<T>, channels: usize, frames: usize) -> Self {
        Self {
            data,
            channels,
            frames,
        }
    }

    /// Allocate an interleaved audio buffer from a fixed-size array acting as a
    /// template for all the channels.
    ///
    /// See [sequential!].
    ///
    /// # Examples
    ///
    /// ```
    /// let buf = audio::buf::Interleaved::from_frames([1.0, 2.0, 3.0, 4.0], 4);
    ///
    /// assert_eq!(buf.frames(), 4);
    /// assert_eq!(buf.channels(), 4);
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

            for f in std::array::IntoIter::new(frames) {
                for _ in 0..channels {
                    data.push(f);
                }
            }

            data
        }
    }

    /// Allocate an interleaved audio buffer from a fixed-size array.
    ///
    /// See [interleaved!].
    ///
    /// # Examples
    ///
    /// ```
    /// let buf = audio::buf::Interleaved::from_array([[1; 4]; 2]);
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
    /// let buf = audio::buf::Interleaved::from_array([[1, 2, 3, 4], [5, 6, 7, 8]]);
    ///
    /// assert_eq!(buf.frames(), 4);
    /// assert_eq!(buf.channels(), 2);
    ///
    /// assert_eq! {
    ///     buf.as_slice(),
    ///     &[1, 5, 2, 6, 3, 7, 4, 8],
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

            // TODO: It would be nice to avoid this heap allocation! Could be
            // done w/ ArrayVec, but we don't want to pull that dependency.
            let mut vecs: Vec<std::array::IntoIter<T, F>> = std::array::IntoIter::new(channels)
                .map(std::array::IntoIter::new)
                .collect();

            for _ in 0..F {
                for v in vecs.iter_mut() {
                    data.extend(v.next());
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
    /// let mut buf = audio::buf::Interleaved::<f32>::with_topology(2, 4);
    ///
    /// for (c, s) in buf.get_mut(0).unwrap().iter_mut().zip(&[1.0, 2.0, 3.0, 4.0]) {
    ///     *c = *s;
    /// }
    ///
    /// for (c, s) in buf.get_mut(1).unwrap().iter_mut().zip(&[1.0, 2.0, 3.0, 4.0]) {
    ///     *c = *s;
    /// }
    ///
    /// buf.resize(3);
    ///
    /// assert_eq!(buf.into_vec(), vec![1.0, 1.0, 2.0, 2.0, 3.0, 3.0])
    /// ```
    pub fn into_vec(self) -> Vec<T> {
        self.data
    }

    /// Access the underlying vector as a slice.
    ///
    /// # Examples
    ///
    /// ```
    /// let buf = audio::buf::Interleaved::<i16>::with_topology(2, 4);
    /// assert_eq!(buf.as_slice(), &[0, 0, 0, 0, 0, 0, 0, 0]);
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
    /// let mut buf = audio::buf::Interleaved::<u32>::with_topology(2, 4);
    /// buf.as_slice_mut().copy_from_slice(&[1, 1, 2, 2, 3, 3, 4, 4]);
    ///
    /// assert_eq! {
    ///     buf.get(0).unwrap(),
    ///     [1u32, 2, 3, 4],
    /// };
    ///
    /// assert_eq! {
    ///     buf.get(1).unwrap(),
    ///     [1u32, 2, 3, 4],
    /// };
    ///
    /// assert_eq!(buf.as_slice(), &[1, 1, 2, 2, 3, 3, 4, 4]);
    /// ```
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        &mut self.data
    }

    /// Get the capacity of the interleaved buffer in number of frames.
    ///
    /// The underlying buffer over-allocates a bit, so this will report the
    /// exact capacity available in the interleaved buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut buf = audio::buf::Interleaved::<f32>::new();
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

    /// Get the number of frames in the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut buf = audio::buf::Interleaved::<f32>::new();
    ///
    /// assert_eq!(buf.frames(), 0);
    /// buf.resize(4);
    /// assert_eq!(buf.frames(), 4);
    /// ```
    pub fn frames(&self) -> usize {
        self.frames
    }

    /// Get the number of channels in the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut buf = audio::buf::Interleaved::<f32>::new();
    ///
    /// assert_eq!(buf.channels(), 0);
    /// buf.resize_channels(2);
    /// assert_eq!(buf.channels(), 2);
    /// ```
    pub fn channels(&self) -> usize {
        self.channels
    }

    /// Resize to the given number of channels in use.
    ///
    /// If the size of the buffer increases as a result, the new channels will
    /// be zeroed. If the size decreases, the channels that falls outside of the
    /// new size will be dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut buf = audio::buf::Interleaved::<f32>::new();
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
    /// ```
    /// let mut buf = audio::buf::Interleaved::<f32>::new();
    ///
    /// assert_eq!(buf.channels(), 0);
    /// assert_eq!(buf.frames(), 0);
    ///
    /// buf.resize_channels(4);
    /// buf.resize(256);
    ///
    /// assert_eq!(buf.channels(), 4);
    /// assert_eq!(buf.frames(), 256);
    ///
    /// {
    ///     let mut chan = buf.get_mut(1).unwrap();
    ///
    ///     assert_eq!(chan.get(127), Some(0.0));
    ///     *chan.get_mut(127).unwrap() = 42.0;
    ///     assert_eq!(chan.get(127), Some(42.0));
    /// }
    ///
    /// buf.resize(128);
    /// assert_eq!(buf.frame(1, 127), Some(42.0));
    ///
    /// buf.resize(256);
    /// assert_eq!(buf.frame(1, 127), Some(42.0));
    ///
    /// buf.resize_channels(2);
    /// assert_eq!(buf.frame(1, 127), Some(42.0));
    ///
    /// buf.resize(64);
    /// assert_eq!(buf.frame(1, 127), None);
    /// ```
    pub fn resize(&mut self, frames: usize)
    where
        T: Sample,
    {
        self.inner_resize(self.channels, frames);
    }

    /// Get a reference to a channel.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut buf = audio::buf::Interleaved::<f32>::with_topology(2, 4);
    ///
    /// for (c, s) in buf.get_mut(0).unwrap().iter_mut().zip(&[1.0, 2.0, 3.0, 4.0]) {
    ///     *c = *s;
    /// }
    ///
    /// for (c, s) in buf.get_mut(1).unwrap().iter_mut().zip(&[5.0, 6.0, 7.0, 8.0]) {
    ///     *c = *s;
    /// }
    ///
    /// assert_eq!(buf.get(0).unwrap().iter().nth(2), Some(3.0));
    /// assert_eq!(buf.get(1).unwrap().iter().nth(2), Some(7.0));
    /// ```
    pub fn get(&self, channel: usize) -> Option<InterleavedRef<'_, T>> {
        if channel < self.channels {
            unsafe {
                let ptr = ptr::NonNull::new_unchecked(self.data.as_ptr() as *mut _);
                let len = self.data.len();
                Some(InterleavedRef::new_unchecked(
                    ptr,
                    len,
                    channel,
                    self.channels,
                ))
            }
        } else {
            None
        }
    }

    /// Helper to access a single frame in a single channel.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut buf = audio::buf::Interleaved::<f32>::with_topology(2, 256);
    ///
    /// assert_eq!(buf.frame(1, 128), Some(0.0));
    /// *buf.frame_mut(1, 128).unwrap() = 1.0;
    /// assert_eq!(buf.frame(1, 128), Some(1.0));
    /// ```
    pub fn frame(&self, channel: usize, frame: usize) -> Option<T>
    where
        T: Copy,
    {
        self.get(channel)?.get(frame)
    }

    /// Get a mutable reference to a channel.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut buf = audio::buf::Interleaved::<f32>::with_topology(2, 4);
    ///
    /// for (c, s) in buf.get_mut(0).unwrap().iter_mut().zip(&[1.0, 2.0, 3.0, 4.0]) {
    ///     *c = *s;
    /// }
    ///
    /// for (c, s) in buf.get_mut(1).unwrap().iter_mut().zip(&[5.0, 6.0, 7.0, 8.0]) {
    ///     *c = *s;
    /// }
    ///
    /// assert_eq!(buf.as_slice(), &[1.0, 5.0, 2.0, 6.0, 3.0, 7.0, 4.0, 8.0]);
    /// ```
    pub fn get_mut(&mut self, channel: usize) -> Option<InterleavedMut<'_, T>> {
        if channel < self.channels {
            unsafe {
                let ptr = ptr::NonNull::new_unchecked(self.data.as_mut_ptr());
                let len = self.data.len();
                Some(InterleavedMut::new_unchecked(
                    ptr,
                    len,
                    channel,
                    self.channels,
                ))
            }
        } else {
            None
        }
    }

    /// Helper to access a single frame in a single channel mutably.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut buf = audio::buf::Interleaved::<f32>::with_topology(2, 256);
    ///
    /// assert_eq!(buf.frame(1, 128), Some(0.0));
    /// *buf.frame_mut(1, 128).unwrap() = 1.0;
    /// assert_eq!(buf.frame(1, 128), Some(1.0));
    /// ```
    pub fn frame_mut(&mut self, channel: usize, frame: usize) -> Option<&mut T> {
        self.get_mut(channel)?.into_mut(frame)
    }

    /// The internal resize function for interleaved channel buffers.
    /// Note: this is safe only because of the `T: Sample` bound. DO NOT REMOVE.
    fn inner_resize(&mut self, channels: usize, frames: usize)
    where
        T: Sample,
    {
        if self.channels == channels && self.frames == frames {
            return;
        }

        self.inner_reserve_cap(frames.saturating_mul(channels));

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

        self.channels = channels;
        self.frames = frames;
    }

    /// Note: this is safe only because of the `T: Sample` bound. DO NOT REMOVE.
    fn inner_reserve_cap(&mut self, new_cap: usize)
    where
        T: Sample,
    {
        let old_cap = self.data.capacity();

        if new_cap > old_cap {
            self.data.reserve(new_cap - old_cap);
            let new_cap = self.data.capacity();

            // Safety: capacity is governed by the underlying vector.
            unsafe {
                ptr::write_bytes(self.data.as_mut_ptr().add(old_cap), 0, new_cap - old_cap);
            }
        }

        // Safety: since we're decreasing the number of frames we're sure
        // that the data for them is already allocated.
        unsafe {
            self.data.set_len(new_cap);
        }
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

impl<T> Interleaved<T>
where
    T: Copy,
{
    /// Construct an iterator over all available channels.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut buf = audio::buf::Interleaved::<f32>::with_topology(2, 4);
    ///
    /// let mut it = buf.iter_mut();
    ///
    /// for (c, f) in it.next().unwrap().iter_mut().zip(&[1.0, 2.0, 3.0, 4.0]) {
    ///     *c = *f;
    /// }
    ///
    /// for (c, f) in it.next().unwrap().iter_mut().zip(&[5.0, 6.0, 7.0, 8.0]) {
    ///     *c = *f;
    /// }
    ///
    /// let channels = buf.iter().collect::<Vec<_>>();
    /// let left = channels[0].iter().collect::<Vec<_>>();
    /// let right = channels[1].iter().collect::<Vec<_>>();
    ///
    /// assert_eq!(left, &[1.0, 2.0, 3.0, 4.0]);
    /// assert_eq!(right, &[5.0, 6.0, 7.0, 8.0]);
    /// ```
    pub fn iter(&self) -> Iter<'_, T> {
        unsafe {
            Iter::new_unchecked(
                ptr::NonNull::new_unchecked(self.data.as_ptr() as *mut _),
                self.data.len(),
                self.channels,
            )
        }
    }

    /// Construct a mutable iterator over all available channels.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut buf = audio::buf::Interleaved::<f32>::with_topology(2, 4);
    ///
    /// let mut it = buf.iter_mut();
    ///
    /// for (c, f) in it.next().unwrap().iter_mut().zip(&[1.0, 2.0, 3.0, 4.0]) {
    ///     *c = *f;
    /// }
    ///
    /// for (c, f) in it.next().unwrap().iter_mut().zip(&[5.0, 6.0, 7.0, 8.0]) {
    ///     *c = *f;
    /// }
    ///
    /// let channels = buf.iter().collect::<Vec<_>>();
    /// let left = channels[0].iter().collect::<Vec<_>>();
    /// let right = channels[1].iter().collect::<Vec<_>>();
    ///
    /// assert_eq!(left, &[1.0, 2.0, 3.0, 4.0]);
    /// assert_eq!(right, &[5.0, 6.0, 7.0, 8.0]);
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        unsafe {
            IterMut::new_unchecked(
                ptr::NonNull::new_unchecked(self.data.as_mut_ptr()),
                self.data.len(),
                self.channels,
            )
        }
    }
}

impl<T> fmt::Debug for Interleaved<T>
where
    T: Copy + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T> cmp::PartialEq for Interleaved<T>
where
    T: Copy + cmp::PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<T> cmp::Eq for Interleaved<T> where T: Copy + cmp::Eq {}

impl<T> cmp::PartialOrd for Interleaved<T>
where
    T: Copy + cmp::PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.iter().partial_cmp(other.iter())
    }
}

impl<T> cmp::Ord for Interleaved<T>
where
    T: Copy + cmp::Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.iter().cmp(other.iter())
    }
}

impl<T> hash::Hash for Interleaved<T>
where
    T: Copy + hash::Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        for channel in self.iter() {
            for f in channel.iter() {
                f.hash(state);
            }
        }
    }
}

impl<T> ExactSizeBuf for Interleaved<T>
where
    T: Copy,
{
    fn frames(&self) -> usize {
        (*self).frames()
    }
}

impl<T> Buf for Interleaved<T>
where
    T: Copy,
{
    type Sample = T;

    type Channel<'a>
    where
        Self::Sample: 'a,
    = InterleavedRef<'a, Self::Sample>;

    type Iter<'a>
    where
        Self::Sample: 'a,
    = Iter<'a, Self::Sample>;

    fn frames_hint(&self) -> Option<usize> {
        Some(self.frames)
    }

    fn channels(&self) -> usize {
        self.channels
    }

    fn get(&self, channel: usize) -> Option<Self::Channel<'_>> {
        InterleavedRef::from_slice(&self.data, channel, self.channels)
    }

    fn iter(&self) -> Self::Iter<'_> {
        (*self).iter()
    }
}

impl<T> ResizableBuf for Interleaved<T>
where
    T: Sample,
{
    fn try_reserve(&mut self, capacity: usize) -> bool {
        self.inner_reserve_cap(capacity);
        true
    }

    fn resize(&mut self, frames: usize) {
        (*self).resize(frames);
    }

    fn resize_topology(&mut self, channels: usize, frames: usize) {
        self.inner_resize(channels, frames);
    }
}

impl<T> BufMut for Interleaved<T>
where
    T: Copy,
{
    type ChannelMut<'a>
    where
        Self::Sample: 'a,
    = InterleavedMut<'a, Self::Sample>;

    type IterMut<'a>
    where
        Self::Sample: 'a,
    = IterMut<'a, Self::Sample>;

    fn get_mut(&mut self, channel: usize) -> Option<Self::ChannelMut<'_>> {
        InterleavedMut::from_slice(&mut self.data, channel, self.channels)
    }

    fn copy_channels(&mut self, from: usize, to: usize) {
        // Safety: We're making sure not to access any mutable buffers which
        // are not initialized.
        unsafe {
            crate::utils::copy_channels_interleaved(
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

impl<'a, T> IntoIterator for &'a Interleaved<T>
where
    T: Copy,
{
    type IntoIter = Iter<'a, T>;
    type Item = <Self::IntoIter as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        (*self).iter()
    }
}

impl<T> InterleavedBuf for Interleaved<T>
where
    T: Copy,
{
    fn as_interleaved(&self) -> &[Self::Sample] {
        self.as_slice()
    }
}

impl<T> InterleavedBufMut for Interleaved<T>
where
    T: Copy,
{
    fn as_interleaved_mut(&mut self) -> &mut [Self::Sample] {
        self.as_slice_mut()
    }

    fn as_interleaved_mut_ptr(&mut self) -> ptr::NonNull<Self::Sample> {
        unsafe { ptr::NonNull::new_unchecked(self.data.as_mut_ptr()) }
    }
}
