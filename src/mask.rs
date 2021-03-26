use crate::bit_set::{BitSet, Bits};

/// The trait for a mask that can be used with [crate::MaskedAudioBuffer].
pub trait Mask: Sized {
    /// The iterator over a mask, indicating all items in the mask.
    type Iter: Iterator<Item = usize>;

    /// Construct a bit set where all elements are set.
    fn full() -> Self;

    /// Test if the given bit is set.
    fn test(&self, index: usize) -> bool;

    /// Set the given bit.
    fn set(&mut self, index: usize);

    /// Clear the given bit.
    fn clear(&mut self, index: usize);

    /// Construct an iterator over a bit set.
    fn iter(&self) -> Self::Iter;
}

impl<T> Mask for BitSet<T>
where
    T: Bits,
{
    type Iter = T::Iter;

    #[inline]
    fn full() -> Self {
        <BitSet<T>>::full()
    }

    #[inline]
    fn test(&self, index: usize) -> bool {
        <BitSet<T>>::test(self, index)
    }

    #[inline]
    fn set(&mut self, index: usize) {
        <BitSet<T>>::set(self, index);
    }

    #[inline]
    fn clear(&mut self, index: usize) {
        <BitSet<T>>::clear(self, index);
    }

    #[inline]
    fn iter(&self) -> Self::Iter {
        <BitSet<T>>::iter(self)
    }
}
