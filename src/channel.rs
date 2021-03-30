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
    pub(crate) inner: RawChannelRef<T>,
    pub(crate) _marker: marker::PhantomData<&'a T>,
}

// Safety: the iterator is simply a container of references to T's, any
// Send/Sync properties are inherited.
unsafe impl<T> Send for Channel<'_, T> where T: Sample + Sync {}
unsafe impl<T> Sync for Channel<'_, T> where T: Sample + Sync {}

impl<'a, T> Channel<'a, T>
where
    T: Sample,
{
    /// Get a reference to a frame.
    pub fn get(&self, frame: usize) -> Option<T> {
        Some(unsafe { *self.inner.frame_ref(frame)? })
    }

    /// Construct an iterator over the current channel.
    pub fn iter(&self) -> ChannelIter<'_, T> {
        ChannelIter {
            inner: self.inner,
            frame: 0,
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
    inner: RawChannelRef<T>,
    frame: usize,
    _marker: marker::PhantomData<&'a T>,
}

// Safety: the iterator is simply a container of references to T's, any
// Send/Sync properties are inherited.
unsafe impl<T> Send for ChannelIter<'_, T> where T: Sample + Sync {}
unsafe impl<T> Sync for ChannelIter<'_, T> where T: Sample + Sync {}

impl<'a, T> Iterator for ChannelIter<'a, T>
where
    T: Sample,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let item = unsafe { &*self.inner.frame_ref(self.frame)? };
        self.frame += 1;
        Some(item)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct RawChannelRef<T> {
    pub(crate) buffer: *const T,
    pub(crate) channel: usize,
    pub(crate) channels: usize,
    pub(crate) frames: usize,
}

impl<T> RawChannelRef<T> {
    /// Get a pointer to the given frame.
    ///
    /// This performs bounds checking to make sure that the frame is in bounds
    /// with the underlying collection.
    #[inline]
    fn frame_ref(self, frame: usize) -> Option<*const T> {
        if frame < self.frames {
            let offset = self.channels * frame + self.channel;
            // Safety: We hold all the parameters necessary to perform bounds
            // checking.
            let frame = unsafe { self.buffer.add(offset) };
            Some(frame)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct RawChannelMut<T> {
    pub(crate) buffer: *mut T,
    pub(crate) channel: usize,
    pub(crate) channels: usize,
    pub(crate) frames: usize,
}

impl<T> RawChannelMut<T> {
    #[inline]
    fn into_ref(self) -> RawChannelRef<T> {
        RawChannelRef {
            buffer: self.buffer as *const _,
            channel: self.channel,
            channels: self.channels,
            frames: self.frames,
        }
    }

    /// Get a mutable pointer to the given frame.
    ///
    /// This performs bounds checking to make sure that the frame is in bounds
    /// with the underlying collection.
    #[inline]
    fn frame_mut(self, frame: usize) -> Option<*mut T> {
        if frame < self.frames {
            let offset = self.channels * frame + self.channel;
            // Safety: We hold all the parameters necessary to perform bounds
            // checking.
            let frame = unsafe { self.buffer.add(offset) };
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
    pub(crate) inner: RawChannelMut<T>,
    pub(crate) _marker: marker::PhantomData<&'a mut T>,
}

// Safety: the iterator is simply a container of mutable references to T's, any
// Send/Sync properties are inherited.
unsafe impl<T> Send for ChannelMut<'_, T> where T: Sample + Send {}
unsafe impl<T> Sync for ChannelMut<'_, T> where T: Sample + Sync {}

impl<'a, T> ChannelMut<'a, T>
where
    T: Sample,
{
    /// Get a reference to a frame.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = rotary::Interleaved::<f32>::with_topology(2, 256);
    ///
    /// let left = buffer.get(0).unwrap();
    ///
    /// assert_eq!(left.get(64), Some(0.0));
    /// assert_eq!(left.get(255), Some(0.0));
    /// assert_eq!(left.get(256), None);
    /// ```
    pub fn get(&self, frame: usize) -> Option<T> {
        // Safety: The lifetime created is associated with the structure that
        // constructed this abstraction.
        unsafe { Some(*self.inner.into_ref().frame_ref(frame)?) }
    }

    /// Get a mutable reference to a frame.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buffer = rotary::Interleaved::<f32>::with_topology(2, 256);
    ///
    /// let mut left = buffer.get_mut(0).unwrap();
    ///
    /// assert_eq!(left.get(64), Some(0.0));
    /// *left.get_mut(64).unwrap() = 1.0;
    /// assert_eq!(left.get(64), Some(1.0));
    /// ```
    pub fn get_mut(&mut self, frame: usize) -> Option<&mut T> {
        // Safety: The lifetime created is associated with the structure that
        // constructed this abstraction.
        unsafe { Some(&mut *self.inner.frame_mut(frame)?) }
    }

    /// Convert the mutable channel into a single mutable frame.
    ///
    /// This is necessary in case you need to build a structure which wraps a
    /// mutable channel and you want it to return the same lifetime as the
    /// mutable channel is associated with.
    ///
    /// # Examples
    ///
    /// This does not build:
    ///
    /// ```rust,compile_fail
    /// struct Foo<'a> {
    ///     left: rotary::channel::ChannelMut<'a, f32>,
    /// }
    ///
    /// impl<'a> Foo<'a> {
    ///     fn get_mut(&mut self, frame: usize) -> Option<&'a mut f32> {
    ///         self.left.get_mut(frame)
    ///     }
    /// }
    ///
    /// let mut buffer = rotary::Interleaved::<f32>::with_topology(2, 256);
    ///
    /// let mut foo = Foo {
    ///     left: buffer.get_mut(0).unwrap(),
    /// };
    ///
    /// *foo.get_mut(64).unwrap() = 1.0;
    /// assert_eq!(buffer.frame(0, 64), Some(1.0));
    /// ```
    ///
    /// ```text
    ///    error[E0495]: cannot infer an appropriate lifetime for autoref due to conflicting requirements
    ///    --> examples\compile-fail.rs:7:19
    ///     |
    ///   7 |         self.left.get_mut(frame)
    ///     |                   ^^^^^^^
    ///     |
    /// ```
    ///
    /// Because if it did, it would be permitted to extract multiple mutable
    /// references to potentially the same frame with the lifetime `'a`, because
    /// `Foo` holds onto the mutable channel.
    ///
    /// The way we can work around this is by allowing a function to consume the
    /// mutable channel. And we can do that with [ChannelMut::into_mut].
    ///
    /// ```rust
    /// # struct Foo<'a> {
    /// #     left: rotary::channel::ChannelMut<'a, f32>,
    /// # }
    ///
    /// impl<'a> Foo<'a> {
    ///     fn into_mut(self, frame: usize) -> Option<&'a mut f32> {
    ///         self.left.into_mut(frame)
    ///     }
    /// }
    ///
    /// let mut buffer = rotary::Interleaved::<f32>::with_topology(2, 256);
    ///
    /// let mut foo = Foo {
    ///     left: buffer.get_mut(0).unwrap(),
    /// };
    ///
    /// *foo.into_mut(64).unwrap() = 1.0;
    /// assert_eq!(buffer.frame(0, 64), Some(1.0));
    /// ```
    pub fn into_mut(self, frame: usize) -> Option<&'a mut T> {
        // Safety: The lifetime created is associated with the structure that
        // constructed this abstraction.
        //
        // We also discard the current channel, which would otherwise allow us
        // to create more `&'a mut T` references, which would be illegal.
        unsafe { Some(&mut *self.inner.frame_mut(frame)?) }
    }

    /// Construct an iterator over the current channel.
    pub fn iter(&self) -> ChannelIter<'_, T> {
        ChannelIter {
            inner: self.inner.into_ref(),
            frame: 0,
            _marker: marker::PhantomData,
        }
    }

    /// Construct a mutable iterator over the current channel.
    pub fn iter_mut(&mut self) -> ChannelIterMut<'_, T> {
        ChannelIterMut {
            inner: self.inner,
            frame: 0,
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
    inner: RawChannelMut<T>,
    frame: usize,
    _marker: marker::PhantomData<&'a mut T>,
}

// Safety: the iterator is simply a container of references to T's, any
// Send/Sync properties are inherited.
unsafe impl<T> Send for ChannelIterMut<'_, T> where T: Sample + Send {}
unsafe impl<T> Sync for ChannelIterMut<'_, T> where T: Sample + Sync {}

impl<'a, T> Iterator for ChannelIterMut<'a, T>
where
    T: Sample,
{
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        let item = unsafe { &mut *self.inner.frame_mut(self.frame)? };
        self.frame += 1;
        Some(item)
    }
}
