//! Traits used to generically describe and operate over slice-like types.

use std::slice;

/// Describes how a buffer can be indexed.
pub trait SliceIndex: Slice
where
    Self: Sized,
{
    /// Get a range out of the given slice.
    fn index<I>(self, index: I) -> Self
    where
        I: slice::SliceIndex<[Self::Item], Output = [Self::Item]>;
}

/// Trait used to operate over a slice.
pub trait Slice
where
    Self: Sized,
{
    /// A single item in the slice.
    type Item: Copy;

    /// Helper to reborrow the items of self.
    fn as_ref(&self) -> &[Self::Item];
}

/// Trait used to operate generically over a mutable slice.
pub trait SliceMut: Slice
where
    Self: Sized,
{
    /// Construct a mutable slice.
    fn as_mut(&mut self) -> &mut [Self::Item];

    /// Get the base mutable pointer.
    fn as_mut_ptr(&mut self) -> *mut Self::Item;
}

impl<T> SliceIndex for &[T]
where
    T: Copy,
{
    fn index<I>(self, index: I) -> Self
    where
        I: slice::SliceIndex<[Self::Item], Output = [Self::Item]>,
    {
        self.get(index).unwrap_or_default()
    }
}

impl<T> Slice for &[T]
where
    T: Copy,
{
    type Item = T;

    fn as_ref(&self) -> &[Self::Item] {
        *self
    }
}

impl<T, const N: usize> Slice for [T; N]
where
    T: Copy,
{
    type Item = T;

    fn as_ref(&self) -> &[Self::Item] {
        &self[..]
    }
}

impl<T, const N: usize> Slice for &[T; N]
where
    T: Copy,
{
    type Item = T;

    fn as_ref(&self) -> &[Self::Item] {
        &self[..]
    }
}

impl<T> SliceIndex for &mut [T]
where
    T: Copy,
{
    fn index<I>(self, index: I) -> Self
    where
        I: slice::SliceIndex<[Self::Item], Output = [Self::Item]>,
    {
        self.get_mut(index).unwrap_or_default()
    }
}

impl<T> Slice for &mut [T]
where
    T: Copy,
{
    type Item = T;

    fn as_ref(&self) -> &[Self::Item] {
        *self
    }
}

impl<T, const N: usize> Slice for &mut [T; N]
where
    T: Copy,
{
    type Item = T;

    fn as_ref(&self) -> &[Self::Item] {
        *self
    }
}

impl<T> SliceMut for &mut [T]
where
    T: Copy,
{
    fn as_mut(&mut self) -> &mut [Self::Item] {
        self
    }

    fn as_mut_ptr(&mut self) -> *mut Self::Item {
        <[T]>::as_mut_ptr(self)
    }
}

impl<T, const N: usize> SliceMut for [T; N]
where
    T: Copy,
{
    fn as_mut(&mut self) -> &mut [Self::Item] {
        self
    }

    fn as_mut_ptr(&mut self) -> *mut Self::Item {
        <[T]>::as_mut_ptr(self)
    }
}

impl<T, const N: usize> SliceMut for &mut [T; N]
where
    T: Copy,
{
    fn as_mut(&mut self) -> &mut [Self::Item] {
        *self
    }

    fn as_mut_ptr(&mut self) -> *mut Self::Item {
        <[T]>::as_mut_ptr(*self)
    }
}
