//! A dynamically sized, multi-channel audio buffer.

use crate::buf::{Buf, BufChannel, BufChannelMut, BufMut};
use crate::mask::Mask;
use crate::masked_dynamic::MaskedDynamic;
use crate::sample::Sample;
use std::cmp;
use std::fmt;
use std::hash;
use std::mem;
use std::ops;
use std::ptr;
use std::slice;

struct MinAllocationFor<T>(std::marker::PhantomData<T>);

/// Helper to calculate the minimum allocation needed for `T`.
impl<T> MinAllocationFor<T> {
    const VALUE: usize = if mem::size_of::<T>() == 1 {
        8
    } else if mem::size_of::<T>() <= 256 {
        4
    } else {
        1
    };
}

/// A dynamically sized, multi-channel audio buffer.
///
/// An audio buffer is constrained to only support sample-apt types. For more
/// information of what this means, see [Sample].
///
/// This kind of buffer stores each channel in its own heap-allocated slice of
/// memory, meaning they can be manipulated more cheaply independently of each
/// other than say [Interleaved][crate::Interleaved] or
/// [Sequential][crate::Sequential]. These would have to re-organize every
/// constituent channel when resizing, while [Dynamic] generally only requires
/// [growing and shrinking][std::alloc::Allocator] of a memory region.
///
/// This kind of buffer is a good choice if you need to
/// [resize][Dynamic::resize] frequently.
pub struct Dynamic<T>
where
    T: Sample,
{
    /// The stored data for each channel.
    data: ptr::NonNull<RawSlice<T>>,
    /// The number of channels stored.
    channels: usize,
    /// The capacity of channels stored.
    channels_cap: usize,
    /// The length of each channel.
    frames: usize,
    /// Allocated capacity of each channel. Each channel is guaranteed to be
    /// filled with as many values as is specified in this capacity.
    frames_cap: usize,
}

impl<T> Dynamic<T>
where
    T: Sample,
{
    const MIN_NON_ZERO_CAP: usize = MinAllocationFor::<T>::VALUE;
    const MIN_NON_ZERO_CHANNELS_CAP: usize = MinAllocationFor::<RawSlice<T>>::VALUE;

    /// Construct a new empty audio buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = rotary::Dynamic::<f32>::new();
    ///
    /// assert_eq!(buffer.frames(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            // Safety: we know that a newly created vector is non-null.
            data: unsafe {
                ptr::NonNull::new_unchecked(mem::ManuallyDrop::new(Vec::new()).as_mut_ptr())
            },
            channels: 0,
            channels_cap: 0,
            frames: 0,
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
    /// let mut buffer = rotary::Dynamic::<f32>::with_topology(4, 256);
    ///
    /// assert_eq!(buffer.frames(), 256);
    /// assert_eq!(buffer.channels(), 4);
    /// ```
    pub fn with_topology(channels: usize, frames: usize) -> Self {
        let data = mem::ManuallyDrop::new(Vec::<RawSlice<T>>::with_capacity(channels)).as_mut_ptr();

        for n in 0..channels {
            // Safety: We just allocated the vector w/ a capacity matching channels.
            unsafe {
                ptr::write(data.add(n), RawSlice::with_capacity(frames));
            }
        }

        Self {
            // Safety: we just initialized the associated array with the
            // expected topology.
            data: unsafe { ptr::NonNull::new_unchecked(data) },
            channels,
            channels_cap: channels,
            frames,
            frames_cap: frames,
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
    /// let mut buffer = rotary::Dynamic::<f32>::from_array([[2.0; 256]; 4]);
    ///
    /// assert_eq!(buffer.frames(), 256);
    /// assert_eq!(buffer.channels(), 4);
    ///
    /// for chan in &buffer {
    ///     assert_eq!(chan, vec![2.0; 256]);
    /// }
    /// ```
    pub fn from_array<const F: usize, const C: usize>(channels: [[T; F]; C]) -> Self {
        return Self {
            // Safety: We just created the box with the data so we know that
            // it's initialized.
            data: unsafe { data_from_array(channels) },
            channels: C,
            channels_cap: C,
            frames: F,
            frames_cap: F,
        };

        #[inline]
        unsafe fn data_from_array<T: Sample, const F: usize, const C: usize>(
            values: [[T; F]; C],
        ) -> ptr::NonNull<RawSlice<T>> {
            let mut data = Vec::with_capacity(C);

            for frames in std::array::IntoIter::new(values) {
                let slice = Box::<[T]>::from(frames);
                let slice = ptr::NonNull::new_unchecked(Box::into_raw(slice) as *mut T);
                data.push(RawSlice { data: slice });
            }

            ptr::NonNull::new_unchecked(mem::ManuallyDrop::new(data).as_mut_ptr())
        }
    }

    /// Convert into a masked audio buffer. The kind of mask needs to be
    /// specified through `M`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let buffer = rotary::dynamic![[2.0; 128]; 4];
    /// let mut buffer = buffer.into_masked::<rotary::BitSet<u128>>();
    ///
    /// buffer.mask(1);
    ///
    /// let mut channels = Vec::new();
    ///
    /// for (n, chan) in buffer.iter_with_channels() {
    ///     channels.push(n);
    ///     assert_eq!(chan, vec![2.0; 128]);
    /// }
    ///
    /// assert_eq!(channels, vec![0, 2, 3]);
    /// ```
    pub fn into_masked<M>(self) -> MaskedDynamic<T, M>
    where
        M: Mask,
    {
        MaskedDynamic::with_buffer(self)
    }

    /// Get the number of frames in the channels of an audio buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = rotary::Dynamic::<f32>::new();
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
    /// let mut buffer = rotary::Dynamic::<f32>::new();
    ///
    /// assert_eq!(buffer.channels(), 0);
    /// buffer.resize_channels(2);
    /// assert_eq!(buffer.channels(), 2);
    /// ```
    pub fn channels(&self) -> usize {
        self.channels
    }

    /// Construct a mutable iterator over all available channels.
    ///
    /// # Examples
    ///
    /// ```
    /// use rand::Rng as _;
    ///
    /// let mut buffer = rotary::Dynamic::<f32>::with_topology(4, 256);
    ///
    /// let all_zeros = vec![0.0; 256];
    ///
    /// for chan in buffer.iter() {
    ///     assert_eq!(chan, &all_zeros[..]);
    /// }
    /// ```
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            // Safety: we're using a trusted length to build the slice.
            iter: unsafe { slice::from_raw_parts(self.data.as_ptr(), self.channels).into_iter() },
            len: self.frames,
        }
    }

    /// Construct a mutable iterator over all available channels.
    ///
    /// # Examples
    ///
    /// ```
    /// use rand::Rng as _;
    ///
    /// let mut buffer = rotary::Dynamic::<f32>::with_topology(4, 256);
    /// let mut rng = rand::thread_rng();
    ///
    /// for chan in buffer.iter_mut() {
    ///     rng.fill(chan);
    /// }
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            // Safety: we're using a trusted length to build the slice.
            iter: unsafe {
                slice::from_raw_parts_mut(self.data.as_ptr(), self.channels).into_iter()
            },
            len: self.frames,
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
    /// let mut buffer = rotary::Dynamic::<f32>::new();
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
    /// # let mut buffer = rotary::Dynamic::<f32>::with_topology(4, 256);
    /// assert_eq!(buffer[1][128], 0.0);
    /// buffer[1][128] = 42.0;
    ///
    /// buffer.resize(64);
    /// assert!(buffer[1].get(128).is_none());
    ///
    /// buffer.resize(256);
    /// assert_eq!(buffer[1][128], 42.0);
    /// ```
    pub fn resize(&mut self, frames: usize) {
        if self.frames_cap < frames {
            let from = self.frames_cap;

            let to = usize::max(from * 2, frames);
            let to = usize::max(Self::MIN_NON_ZERO_CAP, to);

            if self.channels_cap > 0 {
                let additional = to - from;

                for n in 0..self.channels_cap {
                    // Safety: We control the known sizes, so we can guarantee
                    // that the slice is allocated and sized tio exactly `from`.
                    unsafe { (*self.data.as_ptr().add(n)).reserve(from, additional) };
                }
            }

            self.frames_cap = to;
        }

        self.frames = frames;
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
    /// let mut buffer = rotary::Dynamic::<f32>::new();
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
        if channels == self.channels {
            return;
        }

        if channels > self.channels_cap {
            let old_cap = self.channels_cap;
            let new_cap = usize::max(old_cap * 2, channels);
            let new_cap = usize::max(Self::MIN_NON_ZERO_CHANNELS_CAP, new_cap);

            let additional = new_cap - old_cap;

            // Safety: we're constructing the vector from trusted data.
            let data = unsafe {
                let mut data = Vec::from_raw_parts(self.data.as_ptr(), 0, self.channels_cap);
                data.reserve_exact(additional);
                mem::ManuallyDrop::new(data).as_mut_ptr()
            };

            for n in old_cap..new_cap {
                let slice = RawSlice::with_capacity(self.frames_cap);

                // Safety: we control the capacity of channels and have just
                // guranteed above that it is appropriate.
                unsafe {
                    ptr::write(data.add(n), slice);
                }
            }

            // Safety: We just allocated and modified the vector associated w/
            // the pointer.
            self.data = unsafe { ptr::NonNull::new_unchecked(data) };
            self.channels_cap = new_cap;
        }

        debug_assert!(channels <= self.channels_cap);
        self.channels = channels;
    }

    /// Get a reference to the buffer of the given channel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = rotary::Dynamic::<f32>::new();
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
        if channel < self.channels {
            // Safety: We control the length of each channel so we can assert that
            // it is both allocated and initialized up to `len`.
            unsafe { Some((*self.data.as_ptr().add(channel)).as_ref(self.frames)) }
        } else {
            None
        }
    }

    /// Get the given channel or initialize the buffer with the default value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = rotary::Dynamic::<f32>::new();
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
    pub fn get_or_default(&mut self, channel: usize) -> &[T] {
        self.resize_channels(channel + 1);

        // Safety: We initialized the given index just above and we know the
        // trusted length.
        unsafe { (*self.data.as_ptr().add(channel)).as_ref(self.frames) }
    }

    /// Get a mutable reference to the buffer of the given channel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rand::Rng as _;
    ///
    /// let mut buffer = rotary::Dynamic::<f32>::new();
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
        if channel < self.channels {
            // Safety: We control the length of each channel so we can assert that
            // it is both allocated and initialized up to `len`.
            unsafe { Some((*self.data.as_ptr().add(channel)).as_mut(self.frames)) }
        } else {
            None
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
    /// let mut buffer = rotary::Dynamic::<f32>::new();
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
    pub fn get_or_default_mut(&mut self, channel: usize) -> &mut [T] {
        self.resize_channels(channel + 1);

        // Safety: We initialized the given index just above and we know the
        // trusted length.
        unsafe { (*self.data.as_ptr().add(channel)).as_mut(self.frames) }
    }

    /// Convert into a vector of vectors.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = rotary::Dynamic::<f32>::new();
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
        let this = mem::ManuallyDrop::new(self);
        let mut vecs = Vec::with_capacity(this.channels);

        for n in 0..this.channels {
            // Safety: The capacity end lengths are trusted since they're part
            // of the audio buffer.
            unsafe {
                let mut slice = ptr::read(this.data.as_ptr().add(n));

                if m(n) {
                    vecs.push(slice.into_vec(this.frames, this.frames_cap));
                } else {
                    slice.drop_in_place(this.frames_cap);
                    vecs.push(Vec::new());
                }
            }
        }

        // Drop the tail of the channels capacity which is not in use.
        for n in this.channels..this.channels_cap {
            // Safety: The capacity end lengths are trusted since they're part
            // of the audio buffer.
            unsafe {
                let slice = &mut *this.data.as_ptr().add(n);
                slice.drop_in_place(this.frames_cap);
            }
        }

        // Drop the backing vector for all channels.
        //
        // Safety: we trust the capacity provided here.
        unsafe {
            let _ = Vec::from_raw_parts(this.data.as_ptr(), 0, this.channels_cap);
        }

        vecs
    }
}

/// Allocate an audio buffer from a fixed-size array.
///
/// See [dynamic!].
impl<T, const F: usize, const C: usize> From<[[T; F]; C]> for Dynamic<T>
where
    T: Sample,
{
    #[inline]
    fn from(channels: [[T; F]; C]) -> Self {
        Self::from_array(channels)
    }
}

impl<T> fmt::Debug for Dynamic<T>
where
    T: Sample + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T> cmp::PartialEq for Dynamic<T>
where
    T: Sample + cmp::PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<T> cmp::Eq for Dynamic<T> where T: Sample + cmp::Eq {}

impl<T> cmp::PartialOrd for Dynamic<T>
where
    T: Sample + cmp::PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.iter().partial_cmp(other.iter())
    }
}

impl<T> cmp::Ord for Dynamic<T>
where
    T: Sample + cmp::Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.iter().cmp(other.iter())
    }
}

impl<T> hash::Hash for Dynamic<T>
where
    T: Sample + hash::Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        for channel in self.iter() {
            channel.hash(state);
        }
    }
}

impl<'a, T> IntoIterator for &'a Dynamic<T>
where
    T: Sample,
{
    type IntoIter = Iter<'a, T>;
    type Item = <Self::IntoIter as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Dynamic<T>
where
    T: Sample,
{
    type IntoIter = IterMut<'a, T>;
    type Item = <Self::IntoIter as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> ops::Index<usize> for Dynamic<T>
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

impl<T> ops::IndexMut<usize> for Dynamic<T>
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

impl<T> Drop for Dynamic<T>
where
    T: Sample,
{
    fn drop(&mut self) {
        for n in 0..self.channels_cap {
            // Safety: We're being dropped, so there's no subsequent access to
            // the current collection.
            unsafe {
                (*self.data.as_ptr().add(n)).drop_in_place(self.frames_cap);
            }
        }

        // Safety: We trust the length of the underlying array.
        unsafe {
            let _ = Vec::from_raw_parts(self.data.as_ptr(), 0, self.channels_cap);
        }
    }
}

impl<T> Buf<T> for Dynamic<T>
where
    T: Sample,
{
    fn channels(&self) -> usize {
        self.channels
    }

    fn channel(&self, channel: usize) -> BufChannel<'_, T> {
        BufChannel::linear(&self[channel])
    }
}

impl<T> BufMut<T> for Dynamic<T>
where
    T: Sample,
{
    fn channel_mut(&mut self, channel: usize) -> BufChannelMut<'_, T> {
        BufChannelMut::linear(&mut self[channel])
    }

    fn resize(&mut self, frames: usize) {
        Self::resize(self, frames);
    }

    fn resize_topology(&mut self, channels: usize, frames: usize) {
        Self::resize(self, frames);
        self.resize_channels(channels);
    }
}

/// A raw slice.
#[repr(transparent)]
struct RawSlice<T>
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
        let mut channel = Vec::from_raw_parts(self.data.as_ptr(), 0, len);
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
        let _ = Vec::from_raw_parts(self.data.as_ptr(), 0, len);
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
/// Created with [Dynamic::iter_mut].
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
/// Created with [Dynamic::iter_mut].
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
