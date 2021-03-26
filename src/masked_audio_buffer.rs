//! A dynamically sized, multi-channel audio buffer that supports.
//!
//! See [MaskedAudioBuffer] for more information.

use std::ops;

use crate::audio_buffer;
use crate::mask::Mask;
use crate::sample::Sample;

/// A dynamically sized, multi-channel audio buffer that supports masking.
///
/// Masked channels still exist, but they are returned *empty*.
///
/// ```rust
/// use rotary::BitSet;
///
/// let mut buffer = rotary::MaskedAudioBuffer::<f32, BitSet<u128>>::with_topology(2, 256);
///
/// let all_zeros = vec![0.0; 256];
///
/// // Before masking it contains the default value of all zeros.
/// assert_eq!(buffer.get(1), Some(&all_zeros[..]));
///
/// buffer.mask(1);
///
/// // After masking, this channel will always return an empty buffer.
/// assert_eq!(buffer.get(1), Some(&[][..]));
/// ```
///
/// Masked channels will also be skipped by iterators, such as
/// [MaskedAudioBuffer::iter_mut] and
/// [MaskedAudioBuffer::iter_mut_with_channels].
///
/// ```rust
/// use rotary::BitSet;
///
/// let mut buffer = rotary::MaskedAudioBuffer::<f32, BitSet<u128>>::with_topology(2, 256);
///
/// assert_eq!(buffer.iter_mut().count(), 2);
///
/// buffer.mask(1);
///
/// assert_eq!(buffer.iter_mut().count(), 1);
/// ```
pub struct MaskedAudioBuffer<T, M>
where
    T: Sample,
    M: Mask,
{
    buffer: audio_buffer::AudioBuffer<T>,
    mask: M,
}

impl<T, M> MaskedAudioBuffer<T, M>
where
    T: Sample,
    M: Mask,
{
    /// Construct a new empty audio buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::BitSet;
    ///
    /// let mut buffer = rotary::MaskedAudioBuffer::<f32, BitSet<u128>>::new();
    ///
    /// assert_eq!(buffer.frames(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            buffer: audio_buffer::AudioBuffer::new(),
            mask: M::full(),
        }
    }

    /// Allocate an audio buffer with the given topology. A "topology" is a
    /// given number of `channels` and the corresponding number of `frames` in
    /// their buffers.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::BitSet;
    ///
    /// let mut buffer = rotary::MaskedAudioBuffer::<f32, BitSet<u128>>::with_topology(4, 256);
    ///
    /// assert_eq!(buffer.frames(), 256);
    /// assert_eq!(buffer.channels(), 4);
    /// ```
    pub fn with_topology(channels: usize, frames: usize) -> Self {
        Self {
            buffer: audio_buffer::AudioBuffer::with_topology(channels, frames),
            mask: M::full(),
        }
    }

    /// Iterate over the index of all enabled channels.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::BitSet;
    ///
    /// let mut buffer = rotary::MaskedAudioBuffer::<f32, BitSet<u128>>::with_topology(4, 256);
    ///
    /// assert_eq!(vec![0, 1, 2, 3], buffer.unmasked().collect::<Vec<usize>>());
    ///
    /// buffer.mask(1);
    ///
    /// assert_eq!(vec![0, 2, 3], buffer.unmasked().collect::<Vec<usize>>());
    ///
    /// buffer.unmask(1);
    ///
    /// assert_eq!(vec![0, 1, 2, 3], buffer.unmasked().collect::<Vec<usize>>());
    /// ```
    pub fn unmasked(&self) -> Unmasked<M::Iter> {
        Unmasked {
            iter: self.mask.iter(),
            channels: self.buffer.channels(),
        }
    }

    /// Unmask the channel identified by the given `index`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::BitSet;
    ///
    /// let mut buffer = rotary::MaskedAudioBuffer::<f32, BitSet<u128>>::with_topology(4, 256);
    ///
    /// let expected = vec![0.0; 256];
    ///
    /// assert_eq!(buffer.get(0), Some(&expected[..]));
    /// buffer.mask(0);
    /// assert_eq!(buffer.get(0), Some(&[][..]));
    /// buffer.unmask(0);
    /// assert_eq!(buffer.get(0), Some(&expected[..]));
    /// ```
    pub fn unmask(&mut self, index: usize) {
        self.mask.set(index);
    }

    /// Mask the channel identified by the given `index`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::BitSet;
    ///
    /// let mut buffer = rotary::MaskedAudioBuffer::<f32, BitSet<u128>>::with_topology(4, 256);
    ///
    /// let expected = vec![0.0; 256];
    ///
    /// assert_eq!(buffer.get(0), Some(&expected[..]));
    /// buffer.mask(0);
    /// assert_eq!(buffer.get(0), Some(&[][..]));
    /// buffer.unmask(0);
    /// assert_eq!(buffer.get(0), Some(&expected[..]));
    /// ```
    pub fn mask(&mut self, index: usize) {
        self.mask.clear(index);
    }

    /// Build an iterator over all enabled channels.
    ///
    /// # Examples
    ///
    /// ```
    /// use rotary::BitSet;
    ///
    /// let mut buffer = rotary::MaskedAudioBuffer::<f32, BitSet<u128>>::with_topology(4, 256);
    ///
    /// buffer.mask(1);
    ///
    /// let mut channels = Vec::new();
    ///
    /// for (n, chan) in buffer.iter_mut_with_channels() {
    ///     channels.push(n);
    ///
    ///     for f in chan {
    ///         *f = 1.0;
    ///     }
    /// }
    ///
    /// let all_zeros = vec![0.0; 256];
    /// let all_ones = vec![1.0; 256];
    ///
    /// // disabled channels are empty and when re-enabled will contain whatever
    /// // the buffer originally contained.
    /// assert_eq!(&buffer[1], &[][..]);
    ///
    /// buffer.unmask(1);
    ///
    /// assert_eq!(&channels[..], &[0, 2, 3]);
    /// assert_eq!(&buffer[0], &all_ones[..]);
    /// assert_eq!(&buffer[1], &all_zeros[..]);
    /// assert_eq!(&buffer[2], &all_ones[..]);
    /// assert_eq!(&buffer[3], &all_ones[..]);
    /// ```
    pub fn iter_mut_with_channels(&mut self) -> IterMutWithChannels<'_, T, M> {
        IterMutWithChannels {
            slices: self.buffer.iter_mut(),
            iter: self.mask.iter(),
            last: 0,
        }
    }

    /// Construct a mutable iterator over all available channels.
    ///
    /// # Examples
    ///
    /// ```
    /// use rotary::BitSet;
    ///
    /// let mut buffer = rotary::MaskedAudioBuffer::<f32, BitSet<u128>>::with_topology(4, 256);
    ///
    /// let all_zeros = vec![0.0; 256];
    ///
    /// for chan in buffer.iter() {
    ///     assert_eq!(chan, &all_zeros[..]);
    /// }
    ///
    /// buffer.mask(1);
    ///
    /// for chan in buffer.iter_mut() {
    ///     for b in chan {
    ///         *b = 1.0;
    ///     }
    /// }
    ///
    /// let all_ones = vec![1.0; 256];
    ///
    /// for chan in buffer.iter() {
    ///     assert_eq!(chan, &all_ones[..]);
    /// }
    /// ```
    pub fn iter(&self) -> Iter<'_, T, M> {
        Iter {
            slices: self.buffer.iter(),
            iter: self.mask.iter(),
            last: 0,
        }
    }

    /// Construct a mutable iterator over all available channels.
    ///
    /// # Examples
    ///
    /// ```
    /// use rotary::BitSet;
    /// use rand::Rng as _;
    ///
    /// let mut buffer = rotary::MaskedAudioBuffer::<f32, BitSet<u128>>::with_topology(4, 256);
    /// let mut rng = rand::thread_rng();
    ///
    /// for chan in buffer.iter_mut() {
    ///     rng.fill(chan);
    /// }
    /// ```
    ///
    /// With a disabled channel:
    ///
    /// ```
    /// use rotary::BitSet;
    ///
    /// let mut buffer = rotary::MaskedAudioBuffer::<f32, BitSet<u128>>::with_topology(4, 256);
    ///
    /// buffer.mask(1);
    ///
    /// for chan in buffer.iter_mut() {
    ///     for f in chan {
    ///         *f = 1.0;
    ///     }
    /// }
    ///
    /// let all_zeros = vec![0.0; 256];
    /// let all_ones = vec![1.0; 256];
    ///
    /// // disabled channels are empty and when re-enabled will contain whatever
    /// // the buffer originally contained.
    /// assert_eq!(&buffer[1], &[][..]);
    ///
    /// buffer.unmask(1);
    ///
    /// assert_eq!(&buffer[0], &all_ones[..]);
    /// assert_eq!(&buffer[1], &all_zeros[..]);
    /// assert_eq!(&buffer[2], &all_ones[..]);
    /// assert_eq!(&buffer[3], &all_ones[..]);
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<'_, T, M> {
        IterMut {
            slices: self.buffer.iter_mut(),
            iter: self.mask.iter(),
            last: 0,
        }
    }

    /// Get the number of frames in the channels of an audio buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::BitSet;
    ///
    /// let mut buffer = rotary::MaskedAudioBuffer::<f32, BitSet<u128>>::new();
    ///
    /// assert_eq!(buffer.frames(), 0);
    /// buffer.resize(256);
    /// assert_eq!(buffer.frames(), 256);
    /// ```
    pub fn frames(&self) -> usize {
        self.buffer.frames()
    }

    /// Check how many channels there are in the buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::BitSet;
    ///
    /// let mut buffer = rotary::MaskedAudioBuffer::<f32, BitSet<u128>>::new();
    ///
    /// assert_eq!(buffer.channels(), 0);
    /// buffer.resize_channels(2);
    /// assert_eq!(buffer.channels(), 2);
    /// ```
    pub fn channels(&self) -> usize {
        self.buffer.channels()
    }

    /// Set the size of the buffer. The size is the size of each channel's
    /// buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::BitSet;
    ///
    /// let mut buffer = rotary::MaskedAudioBuffer::<f32, BitSet<u128>>::new();
    ///
    /// buffer.resize_channels(4);
    /// buffer.resize(256);
    /// ```
    pub fn resize(&mut self, len: usize) {
        self.buffer.resize(len);
    }

    /// Set the number of channels in use.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::BitSet;
    ///
    /// let mut buffer = rotary::MaskedAudioBuffer::<f32, BitSet<u128>>::new();
    ///
    /// assert_eq!(buffer.channels(), 0);
    /// buffer.resize_channels(4);
    /// assert_eq!(buffer.channels(), 4);
    /// ```
    pub fn resize_channels(&mut self, channels: usize) {
        self.buffer.resize_channels(channels);
    }

    /// Get a reference to the buffer of the given channel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::BitSet;
    ///
    /// let mut buffer = rotary::MaskedAudioBuffer::<f32, BitSet<u128>>::new();
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
        if !self.mask.test(index) {
            return Some(&[]);
        }

        self.buffer.get(index)
    }

    /// Get the given channel or initialize the buffer with the default value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::BitSet;
    ///
    /// let mut buffer = rotary::MaskedAudioBuffer::<f32, BitSet<u128>>::new();
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
        if !self.mask.test(index) {
            return &[];
        }

        self.buffer.get_or_default(index)
    }

    /// Get a mutable reference to the buffer of the given channel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rand::Rng as _;
    /// use rotary::BitSet;
    ///
    /// let mut buffer = rotary::MaskedAudioBuffer::<f32, BitSet<u128>>::new();
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
        if !self.mask.test(index) {
            return Some(&mut []);
        }

        self.buffer.get_mut(index)
    }

    /// Get the given channel or initialize the buffer with the default value.
    ///
    /// If a channel that is out of bound is queried, the buffer will be empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rand::Rng as _;
    /// use rotary::BitSet;
    ///
    /// let mut buffer = rotary::MaskedAudioBuffer::<f32, BitSet<u128>>::new();
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
        if !self.mask.test(index) {
            return &mut [];
        }

        self.buffer.get_or_default_mut(index)
    }

    /// Convert into a vector of vectors.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary::BitSet;
    ///
    /// let mut buffer = rotary::MaskedAudioBuffer::<f32, BitSet<u128>>::new();
    ///
    /// buffer.resize_channels(4);
    /// buffer.resize(512);
    ///
    /// buffer.mask(1);
    ///
    /// let expected = vec![0.0; 512];
    ///
    /// let buffers = buffer.into_vecs();
    /// assert_eq!(buffers.len(), 4);
    /// assert_eq!(buffers[0], &expected[..]);
    /// assert_eq!(buffers[1], &[][..]); // <- disabled channels are empty.
    /// assert_eq!(buffers[2], &expected[..]);
    /// assert_eq!(buffers[3], &expected[..]);
    /// ```
    pub fn into_vecs(self) -> Vec<Vec<T>> {
        let Self { buffer, mask } = self;
        buffer.into_vecs_if(|n| mask.test(n))
    }
}

impl<'a, T, M> IntoIterator for &'a mut MaskedAudioBuffer<T, M>
where
    T: Sample,
    M: Mask,
{
    type IntoIter = IterMut<'a, T, M>;
    type Item = <Self::IntoIter as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, T, M> IntoIterator for &'a MaskedAudioBuffer<T, M>
where
    T: Sample,
    M: Mask,
{
    type IntoIter = Iter<'a, T, M>;
    type Item = <Self::IntoIter as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T, M> ops::Index<usize> for MaskedAudioBuffer<T, M>
where
    T: Sample,
    M: Mask,
{
    type Output = [T];

    fn index(&self, index: usize) -> &Self::Output {
        match self.get(index) {
            Some(slice) => slice,
            None => panic!("index `{}` is not a channel", index),
        }
    }
}

impl<T, M> ops::IndexMut<usize> for MaskedAudioBuffer<T, M>
where
    T: Sample,
    M: Mask,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match self.get_mut(index) {
            Some(slice) => slice,
            None => panic!("index `{}` is not a channel", index,),
        }
    }
}

/// Iterate over all unmasked channels and their corresponding indexes.
///
/// See [MaskedAudioBuffer::unmasked].
pub struct Unmasked<I> {
    channels: usize,
    iter: I,
}

impl<I> Iterator for Unmasked<I>
where
    I: Iterator<Item = usize>,
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.iter.next()?;

        if index < self.channels {
            return Some(index);
        }

        None
    }
}

/// Iterate over all unmasked channels and their corresponding indexes.
///
/// See [MaskedAudioBuffer::iter].
pub struct Iter<'a, T, M>
where
    T: Sample,
    M: Mask,
{
    slices: crate::audio_buffer::Iter<'a, T>,
    iter: M::Iter,
    last: usize,
}

impl<'a, T, M> Iterator for Iter<'a, T, M>
where
    T: Sample,
    M: Mask,
{
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.iter.next()?;
        let offset = index - self.last;
        let buf = self.slices.nth(offset)?;
        self.last = index + 1;
        Some(buf)
    }
}

/// Iterate over all enabled channels and their corresponding indexes.
///
/// See [MaskedAudioBuffer::iter_mut].
pub struct IterMut<'a, T, M>
where
    T: Sample,
    M: Mask,
{
    slices: crate::audio_buffer::IterMut<'a, T>,
    iter: M::Iter,
    last: usize,
}

impl<'a, T, M> Iterator for IterMut<'a, T, M>
where
    T: Sample,
    M: Mask,
{
    type Item = &'a mut [T];

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.iter.next()?;
        let offset = index - self.last;
        let buf = self.slices.nth(offset)?;
        self.last = index + 1;
        Some(buf)
    }
}

/// Iterate over all enabled channels and their corresponding indexes.
///
/// See [MaskedAudioBuffer::iter_mut_with_channels].
pub struct IterMutWithChannels<'a, T, M>
where
    T: Sample,
    M: Mask,
{
    slices: crate::audio_buffer::IterMut<'a, T>,
    iter: M::Iter,
    last: usize,
}

impl<'a, T, M> Iterator for IterMutWithChannels<'a, T, M>
where
    T: Sample,
    M: Mask,
{
    type Item = (usize, &'a mut [T]);

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.iter.next()?;
        let offset = index - self.last;
        let buf = self.slices.nth(offset)?;
        self.last = index + 1;
        Some((index, buf))
    }
}
