//! A dynamically sized, multi-channel audio buffer.

use crate::sample::Sample;
use std::cmp;
use std::fmt;
use std::hash;
use std::mem;
use std::ops;
use std::ptr;
use std::slice;

/// A multi-channel AudioBuffer audio buffer for the given type.
///
/// A AudioBuffer audio buffer is constrained to only support sample-apt types,
/// which have the following properties allowing the container to work more
/// efficiently:
///
/// * The type `T` does not need to be dropped.
/// * The type `T` can safely be initialized with the all-zeros bit pattern.
pub struct AudioBuffer<T>
where
    T: Sample,
{
    /// The stored data for each channel.
    // TODO: Figure out how to remove `pub(crate)` by creating safer APIs for
    // iterating over the available channels.
    channels: Vec<RawSlice<T>>,
    /// The length of each channel.
    frames_len: usize,
    /// Allocated capacity of each channel. Each channel is guaranteed to be
    /// filled with as many values as is specified in this capacity.
    frames_cap: usize,
}

impl<T> AudioBuffer<T>
where
    T: Sample,
{
    const MIN_NON_ZERO_CAP: usize = if mem::size_of::<T>() == 1 {
        8
    } else if mem::size_of::<T>() <= 256 {
        4
    } else {
        1
    };

    /// Construct a new empty audio buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = rotary::AudioBuffer::<f32>::new();
    ///
    /// assert_eq!(buffer.frames(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            channels: Vec::new(),
            frames_len: 0,
            frames_cap: 0,
        }
    }

    /// Allocate an audio buffer with the given topology. A "topology" is a
    /// given number of `channels` and the corresponding number of `frames` in
    /// their buffers.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = rotary::AudioBuffer::<f32>::with_topology(4, 256);
    ///
    /// assert_eq!(buffer.frames(), 256);
    /// assert_eq!(buffer.channels(), 4);
    /// ```
    pub fn with_topology(channels: usize, frames: usize) -> Self {
        let mut channels = Vec::with_capacity(channels);

        for _ in 0..channels.capacity() {
            channels.push(RawSlice::with_capacity(frames));
        }

        Self {
            channels,
            frames_len: frames,
            frames_cap: frames,
        }
    }

    /// Get the number of frames in the channels of an audio buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = rotary::AudioBuffer::<f32>::new();
    ///
    /// assert_eq!(buffer.frames(), 0);
    /// buffer.resize(256);
    /// assert_eq!(buffer.frames(), 256);
    /// ```
    pub fn frames(&self) -> usize {
        self.frames_len
    }

    /// Check how many channels there are in the buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = rotary::AudioBuffer::<f32>::new();
    ///
    /// assert_eq!(buffer.channels(), 0);
    /// buffer.resize_channels(2);
    /// assert_eq!(buffer.channels(), 2);
    /// ```
    pub fn channels(&self) -> usize {
        self.channels.len()
    }

    /// Construct a mutable iterator over all available channels.
    ///
    /// # Examples
    ///
    /// ```
    /// use rand::Rng as _;
    ///
    /// let mut buffer = rotary::AudioBuffer::<f32>::with_topology(4, 256);
    ///
    /// let all_zeros = vec![0.0; 256];
    ///
    /// for chan in buffer.iter() {
    ///     assert_eq!(chan, &all_zeros[..]);
    /// }
    /// ```
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            iter: self.channels.iter(),
            len: self.frames_len,
        }
    }

    /// Construct a mutable iterator over all available channels.
    ///
    /// # Examples
    ///
    /// ```
    /// use rand::Rng as _;
    ///
    /// let mut buffer = rotary::AudioBuffer::<f32>::with_topology(4, 256);
    /// let mut rng = rand::thread_rng();
    ///
    /// for chan in buffer.iter_mut() {
    ///     rng.fill(chan);
    /// }
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            iter: self.channels.iter_mut(),
            len: self.frames_len,
        }
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
    /// let mut buffer = rotary::AudioBuffer::<f32>::new();
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
    /// Decreasing and increasing the size will not touch a buffer that has
    /// already been allocated.
    ///
    /// ```rust
    /// # let mut buffer = rotary::AudioBuffer::<f32>::with_topology(4, 256);
    /// assert_eq!(buffer[1][128], 0.0);
    /// buffer[1][128] = 42.0;
    ///
    /// buffer.resize(64);
    /// assert!(buffer[1].get(128).is_none());
    ///
    /// buffer.resize(256);
    /// assert_eq!(buffer[1][128], 42.0);
    /// ```
    pub fn resize(&mut self, len: usize) {
        if self.frames_cap < len {
            if self.channels.is_empty() {
                let cap = usize::max(self.frames_cap * 2, len);
                self.frames_cap = usize::max(Self::MIN_NON_ZERO_CAP, cap);
            } else {
                let from = self.frames_cap;
                let to = usize::max(from * 2, len);
                let to = usize::max(Self::MIN_NON_ZERO_CAP, to);

                let additional = to - from;

                for slice in &mut self.channels {
                    // Safety: We control the known sizes, so we can guarantee
                    // that the slice is allocated and sized tio exactly `from`.
                    unsafe { slice.reserve(from, additional) };
                }

                self.frames_cap = to;
            }
        }

        self.frames_len = len;
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
    /// let mut buffer = rotary::AudioBuffer::<f32>::new();
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
        if channels < self.channels.len() {
            // Drop the tail.
            for slice in &mut self.channels[channels..] {
                // Safety: We immediately set the length the the channels
                // buffer after dropping the slice.
                unsafe {
                    slice.drop_in_place(self.frames_cap);
                }
            }

            // Safety: We specifically only update the length of the number of
            // channels since there is no need to re-allocate.
            unsafe {
                self.channels.set_len(channels);
            }
        } else if channels > self.channels.len() {
            let extra = channels - self.channels.len();

            for _ in 0..extra {
                self.channels.push(RawSlice::with_capacity(self.frames_cap));
            }
        }
    }

    /// Get a reference to the buffer of the given channel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = rotary::AudioBuffer::<f32>::new();
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
    pub fn get(&self, index: usize) -> Option<&[T]> {
        // Safety: We control the length of each channel so we can assert that
        // it is both allocated and initialized up to `len`.
        unsafe {
            let slice = self.channels.get(index)?;
            Some(slice.as_ref(self.frames_len))
        }
    }

    /// Get the given channel or initialize the buffer with the default value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = rotary::AudioBuffer::<f32>::new();
    ///
    /// buffer.resize(256);
    ///
    /// let expected = vec![0f32; 256];
    ///
    /// assert_eq!(buffer.get_or_default(0), &expected[..]);
    /// assert_eq!(buffer.get_or_default(1), &expected[..]);
    ///
    /// assert_eq!(buffer.channels(), 2);
    /// ```
    pub fn get_or_default(&mut self, index: usize) -> &[T] {
        if index >= self.channels.len() {
            for _ in 0..=(index - self.channels.len()) {
                self.channels.push(RawSlice::with_capacity(self.frames_cap));
            }
        }

        // Safety: We initialized the given index just above and we know the
        // trusted length.
        unsafe { self.channels.get_unchecked(index).as_ref(self.frames_len) }
    }

    /// Get a mutable reference to the buffer of the given channel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rand::Rng as _;
    ///
    /// let mut buffer = rotary::AudioBuffer::<f32>::new();
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
    pub fn get_mut(&mut self, index: usize) -> Option<&mut [T]> {
        // Safety: We control the length of each channel so we can assert that
        // it is both allocated and initialized up to `len`.
        unsafe {
            let slice = self.channels.get_mut(index)?;
            Some(slice.as_mut(self.frames_len))
        }
    }

    /// Get the given channel or initialize the buffer with the default value.
    ///
    /// If a channel that is out of bound is queried, the buffer will be empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rand::Rng as _;
    ///
    /// let mut buffer = rotary::AudioBuffer::<f32>::new();
    ///
    /// buffer.resize(256);
    ///
    /// let mut rng = rand::thread_rng();
    ///
    /// rng.fill(buffer.get_or_default_mut(0));
    /// rng.fill(buffer.get_or_default_mut(1));
    ///
    /// assert_eq!(buffer.channels(), 2);
    /// ```
    pub fn get_or_default_mut(&mut self, index: usize) -> &mut [T] {
        if index >= self.channels.len() {
            for _ in 0..=(index - self.channels.len()) {
                self.channels.push(RawSlice::with_capacity(self.frames_cap));
            }
        }

        // Safety: We initialized the given index just above and we know the
        // trusted length.
        unsafe {
            self.channels
                .get_unchecked_mut(index)
                .as_mut(self.frames_len)
        }
    }

    /// Convert into a vector of vectors.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = rotary::AudioBuffer::<f32>::new();
    /// buffer.resize_channels(4);
    /// buffer.resize(512);
    ///
    /// let expected = vec![0.0; 512];
    ///
    /// let buffers = buffer.into_vecs();
    /// assert_eq!(buffers.len(), 4);
    /// assert_eq!(buffers[0], &expected[..]);
    /// assert_eq!(buffers[1], &expected[..]);
    /// assert_eq!(buffers[2], &expected[..]);
    /// assert_eq!(buffers[3], &expected[..]);
    /// ```
    pub fn into_vecs(self) -> Vec<Vec<T>> {
        self.into_vecs_if(|_| true)
    }

    pub(crate) fn into_vecs_if(self, mut m: impl FnMut(usize) -> bool) -> Vec<Vec<T>> {
        let mut this = mem::ManuallyDrop::new(self);
        let mut vecs = Vec::with_capacity(this.channels.len());

        let len = this.frames_len;
        let cap = this.frames_cap;
        let channels = std::mem::take(&mut this.channels);

        for (n, mut slice) in channels.into_iter().enumerate() {
            // Safety: The capacity end lengths are trusted since they're part
            // of the audio buffer.
            unsafe {
                if m(n) {
                    vecs.push(slice.into_vec(len, cap));
                } else {
                    slice.drop_in_place(len);
                    vecs.push(Vec::new());
                }
            }
        }

        vecs
    }
}

impl<T> fmt::Debug for AudioBuffer<T>
where
    T: Sample + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T> cmp::PartialEq for AudioBuffer<T>
where
    T: Sample + cmp::PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<T> cmp::Eq for AudioBuffer<T> where T: Sample + cmp::Eq {}

impl<T> cmp::PartialOrd for AudioBuffer<T>
where
    T: Sample + cmp::PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.iter().partial_cmp(other.iter())
    }
}

impl<T> cmp::Ord for AudioBuffer<T>
where
    T: Sample + cmp::Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.iter().cmp(other.iter())
    }
}

impl<T> hash::Hash for AudioBuffer<T>
where
    T: Sample + hash::Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        for channel in self.iter() {
            channel.hash(state);
        }
    }
}

/// Allocate an audio buffer from a fixed-size array.
///
/// See [crate::macros::audio_buffer!].
impl<T, const F: usize, const C: usize> From<[[T; F]; C]> for AudioBuffer<T>
where
    T: Sample,
{
    #[inline]
    fn from(channels: [[T; F]; C]) -> Self {
        return Self {
            channels: channels_from_array(channels),
            frames_cap: F,
            frames_len: F,
        };

        #[inline]
        fn channels_from_array<T: Sample, const F: usize, const C: usize>(
            values: [[T; F]; C],
        ) -> Vec<RawSlice<T>> {
            let mut channels = Vec::with_capacity(C);

            for frames in std::array::IntoIter::new(values) {
                let data = Box::<[T]>::from(frames);

                // Safety: We just created the box with the data.
                let data = unsafe { ptr::NonNull::new_unchecked(Box::into_raw(data) as *mut T) };

                channels.push(RawSlice { data });
            }

            channels
        }
    }
}

impl<'a, T> IntoIterator for &'a AudioBuffer<T>
where
    T: Sample,
{
    type IntoIter = Iter<'a, T>;
    type Item = <Self::IntoIter as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut AudioBuffer<T>
where
    T: Sample,
{
    type IntoIter = IterMut<'a, T>;
    type Item = <Self::IntoIter as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> ops::Index<usize> for AudioBuffer<T>
where
    T: Sample,
{
    type Output = [T];

    fn index(&self, index: usize) -> &Self::Output {
        match self.get(index) {
            Some(slice) => slice,
            None => panic!("index `{}` is not a channel", index),
        }
    }
}

impl<T> ops::IndexMut<usize> for AudioBuffer<T>
where
    T: Sample,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match self.get_mut(index) {
            Some(slice) => slice,
            None => panic!("index `{}` is not a channel", index,),
        }
    }
}

impl<T> Drop for AudioBuffer<T>
where
    T: Sample,
{
    fn drop(&mut self) {
        for channel in &mut self.channels {
            // Safety: We're being dropped, so there's no subsequent access to
            // the current collection.
            unsafe {
                channel.drop_in_place(self.frames_cap);
            }
        }
    }
}

/// A raw slice.
pub(crate) struct RawSlice<T>
where
    T: Sample,
{
    data: ptr::NonNull<T>,
}

impl<T> RawSlice<T>
where
    T: Sample,
{
    /// Construct a new raw slice with the given capacity.
    fn with_capacity(cap: usize) -> Self {
        // Safety: the type constrain of `T` guarantees that an all-zeros bit
        // pattern is legal.
        unsafe {
            let mut data = Vec::with_capacity(cap);
            ptr::write_bytes(data.as_mut_ptr(), 0, cap);
            let data = ptr::NonNull::new_unchecked(mem::ManuallyDrop::new(data).as_mut_ptr());
            Self { data }
        }
    }

    /// Resize the slice in place by reserving `additional` more elements in it.
    ///
    /// # Safety
    ///
    /// The provided `len` must watch the length for which it was allocated.
    /// This will change the underlying allocation, so subsequent calls must
    /// provide the new length of `len + extra`.
    unsafe fn reserve(&mut self, len: usize, additional: usize) {
        let mut channel = Vec::from_raw_parts(self.data.as_ptr(), len, len);
        channel.reserve_exact(additional);

        // Safety: the type constrain of `T` guarantees that an all-zeros bit pattern is legal.
        ptr::write_bytes(channel.as_mut_ptr().add(len), 0, additional);
        self.data = ptr::NonNull::new_unchecked(mem::ManuallyDrop::new(channel).as_mut_ptr());
    }

    /// Get the raw slice as a slice.
    ///
    /// # Safety
    ///
    /// The incoming len must represent a valid slice of initialized data.
    pub(crate) unsafe fn as_ref(&self, len: usize) -> &[T] {
        slice::from_raw_parts(self.data.as_ptr() as *const _, len)
    }

    /// Get the raw slice as a mutable slice.
    ///
    /// # Safety
    ///
    /// The incoming len must represent a valid slice of initialized data.
    pub(crate) unsafe fn as_mut(&mut self, len: usize) -> &mut [T] {
        slice::from_raw_parts_mut(self.data.as_ptr(), len)
    }

    /// Drop the slice in place.
    ///
    /// # Safety
    ///
    /// The provided `len` must match the allocated length of the raw slice.
    ///
    /// After calling drop, the slice must not be used every again because the
    /// data it is pointing to have been dropped.
    unsafe fn drop_in_place(&mut self, len: usize) {
        let _ = Vec::from_raw_parts(self.data.as_ptr(), len, len);
    }

    /// Convert into a vector.
    ///
    /// # Safety
    ///
    /// The provided `len` and `cap` must match the ones used when allocating
    /// the slice.
    ///
    /// The underlying slices must be dropped and forgotten after this
    /// operation.
    pub(crate) unsafe fn into_vec(self, len: usize, cap: usize) -> Vec<T> {
        Vec::from_raw_parts(self.data.as_ptr(), len, cap)
    }
}

/// A mutable iterator over the channels in the buffer.
///
/// Created with [AudioBuffer::iter_mut].
pub struct Iter<'a, T>
where
    T: Sample,
{
    iter: slice::Iter<'a, RawSlice<T>>,
    len: usize,
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: Sample,
{
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        let buf = self.iter.next()?;
        Some(unsafe { buf.as_ref(self.len) })
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let buf = self.iter.nth(n)?;
        Some(unsafe { buf.as_ref(self.len) })
    }
}

/// A mutable iterator over the channels in the buffer.
///
/// Created with [AudioBuffer::iter_mut].
pub struct IterMut<'a, T>
where
    T: Sample,
{
    iter: slice::IterMut<'a, RawSlice<T>>,
    len: usize,
}

impl<'a, T> Iterator for IterMut<'a, T>
where
    T: Sample,
{
    type Item = &'a mut [T];

    fn next(&mut self) -> Option<Self::Item> {
        let buf = self.iter.next()?;
        Some(unsafe { buf.as_mut(self.len) })
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let buf = self.iter.nth(n)?;
        Some(unsafe { buf.as_mut(self.len) })
    }
}
