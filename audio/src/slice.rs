//! Traits used to generically describe and operate over slice-like types.
//!
//! This is used in for example [LinearChannel][crate::channel::LinearChannel] to allow
//! it to abstract over its content.

#![allow(clippy::len_without_is_empty)]

use core::ptr;

/// Describes how a buffer can be indexed.
pub trait SliceIndex: Slice
where
    Self: Sized,
{
    /// Get a range out of the given slice.
    #[doc(hidden)]
    fn index_from(self, index: usize) -> Self;

    /// Index to the given index.
    #[doc(hidden)]
    fn index_to(self, index: usize) -> Self;

    /// Index to the given index.
    #[doc(hidden)]
    fn index_full(self, from: usize, to: usize) -> Self;
}

/// Trait used to operate over a slice.
pub trait Slice
where
    Self: Sized,
{
    /// A single item in the slice.
    type Item: Copy;

    /// Get the length of the slice.
    #[doc(hidden)]
    fn len(&self) -> usize;

    /// Helper to reborrow the items of self.
    #[doc(hidden)]
    fn as_ref(&self) -> &[Self::Item];

    /// Get the pointer to the first element.
    #[doc(hidden)]
    fn as_ptr(&self) -> ptr::NonNull<Self::Item>;
}

/// Trait used to operate generically over a mutable slice.
pub trait SliceMut: Slice
where
    Self: Sized,
{
    /// Construct a mutable slice.
    fn as_mut(&mut self) -> &mut [Self::Item];

    /// Get the base mutable pointer.
    fn as_mut_ptr(&mut self) -> ptr::NonNull<Self::Item>;
}

impl<T> SliceIndex for &[T]
where
    T: Copy,
{
    #[inline]
    fn index_from(self, index: usize) -> Self {
        self.get(index..).unwrap_or_default()
    }

    #[inline]
    fn index_to(self, index: usize) -> Self {
        self.get(..index).unwrap_or_default()
    }

    #[inline]
    fn index_full(self, from: usize, to: usize) -> Self {
        self.get(from..to).unwrap_or_default()
    }
}

impl<T> Slice for &[T]
where
    T: Copy,
{
    type Item = T;

    #[inline]
    fn len(&self) -> usize {
        <[T]>::len(self)
    }

    #[inline]
    fn as_ref(&self) -> &[Self::Item] {
        self
    }

    #[inline]
    fn as_ptr(&self) -> ptr::NonNull<Self::Item> {
        unsafe { ptr::NonNull::new_unchecked(<[T]>::as_ptr(&self[..]) as *mut _) }
    }
}

impl<T, const N: usize> Slice for [T; N]
where
    T: Copy,
{
    type Item = T;

    #[inline]
    fn len(&self) -> usize {
        N
    }

    #[inline]
    fn as_ref(&self) -> &[Self::Item] {
        &self[..]
    }

    #[inline]
    fn as_ptr(&self) -> ptr::NonNull<Self::Item> {
        unsafe { ptr::NonNull::new_unchecked(<[T]>::as_ptr(self) as *mut _) }
    }
}

impl<T, const N: usize> Slice for &[T; N]
where
    T: Copy,
{
    type Item = T;

    #[inline]
    fn len(&self) -> usize {
        N
    }

    #[inline]
    fn as_ref(&self) -> &[Self::Item] {
        &self[..]
    }

    #[inline]
    fn as_ptr(&self) -> ptr::NonNull<Self::Item> {
        unsafe { ptr::NonNull::new_unchecked(<[T]>::as_ptr(&self[..]) as *mut _) }
    }
}

impl<T> SliceIndex for &mut [T]
where
    T: Copy,
{
    #[inline]
    fn index_from(self, index: usize) -> Self {
        self.get_mut(index..).unwrap_or_default()
    }

    #[inline]
    fn index_to(self, index: usize) -> Self {
        self.get_mut(..index).unwrap_or_default()
    }

    #[inline]
    fn index_full(self, from: usize, to: usize) -> Self {
        self.get_mut(from..to).unwrap_or_default()
    }
}

impl<T> Slice for &mut [T]
where
    T: Copy,
{
    type Item = T;

    #[inline]
    fn len(&self) -> usize {
        <[T]>::len(self)
    }

    #[inline]
    fn as_ref(&self) -> &[Self::Item] {
        &self[..]
    }

    #[inline]
    fn as_ptr(&self) -> ptr::NonNull<Self::Item> {
        unsafe { ptr::NonNull::new_unchecked(<[T]>::as_ptr(&self[..]) as *mut _) }
    }
}

impl<T, const N: usize> Slice for &mut [T; N]
where
    T: Copy,
{
    type Item = T;

    #[inline]
    fn len(&self) -> usize {
        N
    }

    #[inline]
    fn as_ref(&self) -> &[Self::Item] {
        &self[..]
    }

    #[inline]
    fn as_ptr(&self) -> ptr::NonNull<Self::Item> {
        unsafe { ptr::NonNull::new_unchecked(<[T]>::as_ptr(&self[..]) as *mut _) }
    }
}

impl<T> SliceMut for &mut [T]
where
    T: Copy,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [Self::Item] {
        self
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> ptr::NonNull<Self::Item> {
        unsafe { ptr::NonNull::new_unchecked(<[T]>::as_mut_ptr(self)) }
    }
}

impl<T, const N: usize> SliceMut for [T; N]
where
    T: Copy,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [Self::Item] {
        self
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> ptr::NonNull<Self::Item> {
        unsafe { ptr::NonNull::new_unchecked(<[T]>::as_mut_ptr(&mut self[..])) }
    }
}

impl<T, const N: usize> SliceMut for &mut [T; N]
where
    T: Copy,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [Self::Item] {
        &mut self[..]
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> ptr::NonNull<Self::Item> {
        unsafe { ptr::NonNull::new_unchecked(<[T]>::as_mut_ptr(&mut self[..])) }
    }
}
