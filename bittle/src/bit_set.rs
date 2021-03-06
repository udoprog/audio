//! A fixed size bit set.

use crate::mask::Mask;
use std::cmp;
use std::fmt;
use std::hash;

/// A fixed size bit set.
///
/// # Examples
///
/// ```rust
/// let mut set = bittle::BitSet::<u128>::empty();
///
/// assert!(!set.test(1));
/// set.set(1);
/// assert!(set.test(1));
/// set.clear(1);
/// assert!(!set.test(1));
/// ```
///
/// The bit set can also use arrays as its backing storage.
///
/// ```rust
/// let mut set = bittle::BitSet::<[u64; 16]>::empty();
///
/// assert!(!set.test(172));
/// set.set(172);
/// assert!(set.test(172));
/// set.clear(172);
/// assert!(!set.test(172));
/// ```
///
/// Two bit sets of different kinds can be compared to each other.
///
/// ```rust
/// let mut a = bittle::BitSet::<[u64; 2]>::empty();
/// let mut b = bittle::BitSet::<u128>::empty();
///
/// assert_eq!(a, b);
///
/// a.set(111);
/// assert_ne!(a, b);
///
/// b.set(111);
/// assert_eq!(a, b);
/// ```
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct BitSet<T>
where
    T: Bits,
{
    bits: T,
}

impl<T> BitSet<T>
where
    T: Bits,
{
    /// Construct a bit set from an array.
    pub fn from_array<const N: usize>(items: [usize; N]) -> Self {
        let mut set = Self::empty();

        for n in std::array::IntoIter::new(items) {
            set.set(n);
        }

        set
    }

    /// Construct a new bit set that is empty, where no element is set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let set = bittle::BitSet::<u128>::empty();
    ///
    /// assert_eq!(set.iter().collect::<Vec<_>>(), vec![])
    /// ```
    pub fn empty() -> Self {
        Self { bits: T::EMPTY }
    }

    /// Construct a new bit set that is full, where every single element
    /// possible is set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let set = bittle::BitSet::<u128>::full();
    ///
    /// assert_eq!(set.iter().collect::<Vec<_>>(), (0..128usize).collect::<Vec<_>>())
    /// ```
    pub fn full() -> Self {
        Self { bits: T::FULL }
    }

    /// Test if the given bit is set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut set = bittle::BitSet::<u128>::full();
    ///
    /// assert!(set.test(0));
    /// assert!(set.test(1));
    /// assert!(set.test(127));
    ///
    /// set.clear(1);
    ///
    /// assert!(set.test(0));
    /// assert!(!set.test(1));
    /// assert!(set.test(127));
    /// ```
    pub fn test(&self, index: usize) -> bool {
        self.bits.test(index)
    }

    /// Set the given bit.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut set = bittle::BitSet::<u128>::full();
    ///
    /// assert!(set.test(0));
    /// assert!(set.test(1));
    /// assert!(set.test(127));
    ///
    /// set.clear(1);
    ///
    /// assert!(set.test(0));
    /// assert!(!set.test(1));
    /// assert!(set.test(127));
    ///
    /// set.set(1);
    ///
    /// assert!(set.test(0));
    /// assert!(set.test(1));
    /// assert!(set.test(127));
    /// ```
    pub fn set(&mut self, index: usize) {
        self.bits.set(index);
    }

    /// Clear the given bit.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut set = bittle::BitSet::<u128>::full();
    ///
    /// assert!(set.test(0));
    /// assert!(set.test(1));
    /// assert!(set.test(127));
    ///
    /// set.clear(1);
    ///
    /// assert!(set.test(0));
    /// assert!(!set.test(1));
    /// assert!(set.test(127));
    ///
    /// set.set(1);
    ///
    /// assert!(set.test(0));
    /// assert!(set.test(1));
    /// assert!(set.test(127));
    /// ```
    pub fn clear(&mut self, index: usize) {
        self.bits.clear(index);
    }

    /// Construct an iterator over a bit set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut set = bittle::BitSet::<u128>::empty();
    ///
    /// set.set(3);
    /// set.set(7);
    ///
    /// assert_eq!(set.iter().collect::<Vec<_>>(), vec![3, 7]);
    /// ```
    ///
    /// A larger bit set:
    ///
    /// ```rust
    /// use bittle::Mask as _;
    ///
    /// let mut set = bittle::BitSet::<[u32; 4]>::empty();
    ///
    /// set.set(4);
    /// set.set(63);
    /// set.set(71);
    ///
    /// assert_eq!(set.iter().collect::<Vec<_>>(), vec![4, 63, 71]);
    /// ```
    pub fn iter(&self) -> T::Iter {
        self.bits.iter()
    }
}

impl<T> fmt::Debug for BitSet<T>
where
    T: Bits,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T, U> cmp::PartialEq<BitSet<U>> for BitSet<T>
where
    T: Bits,
    U: Bits,
{
    fn eq(&self, other: &BitSet<U>) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<T> cmp::Eq for BitSet<T> where T: Bits {}

impl<T, U> cmp::PartialOrd<BitSet<U>> for BitSet<T>
where
    T: Bits,
    U: Bits,
{
    fn partial_cmp(&self, other: &BitSet<U>) -> Option<cmp::Ordering> {
        self.iter().partial_cmp(other.iter())
    }
}

impl<T> cmp::Ord for BitSet<T>
where
    T: Bits,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.iter().cmp(other.iter())
    }
}

impl<T> hash::Hash for BitSet<T>
where
    T: Bits + hash::Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        for index in self.iter() {
            index.hash(state);
        }
    }
}

impl<T> IntoIterator for BitSet<T>
where
    T: Bits,
{
    type IntoIter = T::Iter;
    type Item = <Self::IntoIter as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An iterator over a bit bits. Created through [BitSet::iter].
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Iter<T>
where
    T: Bits + Number,
{
    bits: T,
}

impl<T> Iterator for Iter<T>
where
    T: Bits + Number,
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bits.is_zero() {
            return None;
        }

        let index = self.bits.trailing_zeros();
        self.bits.clear(index);
        Some(index)
    }
}

/// An iterator over a bit bits. Created through [BitSet::iter].
#[derive(Clone, Copy)]
pub struct ArrayIter<T, const N: usize>
where
    T: Bits + Number,
{
    bits: [T; N],
    o: usize,
}

impl<T, const N: usize> Iterator for ArrayIter<T, N>
where
    T: Bits + Number,
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(bits) = self.bits.get_mut(self.o) {
            if !bits.is_zero() {
                let index = bits.trailing_zeros();
                bits.clear(index);
                return Some(self.o * T::BITS + index);
            }

            self.o += 1;
        }

        None
    }
}

/// Basic numerical traits for the plumbing of a bit set.
pub trait Number: Bits {
    /// How many bits there are in this number.
    const BITS: usize = std::mem::size_of::<Self>() * 8;

    /// Number of trailing zeros.
    fn trailing_zeros(self) -> usize;

    /// Test if the number is zero.
    fn is_zero(self) -> bool;
}

/// The trait for a bit pattern.
pub trait Bits: Sized + Copy {
    /// The iterator over this bit pattern.
    ///
    /// See [BitSet::iter].
    type Iter: Iterator<Item = usize>;

    /// Bit-pattern for an empty bit pattern.
    ///
    /// See [BitSet::empty].
    const EMPTY: Self;

    /// Bit-pattern for a full bit pattern.
    ///
    /// See [BitSet::full].
    const FULL: Self;

    /// Test if the given bit is set.
    ///
    /// See [BitSet::test].
    fn test(&self, index: usize) -> bool;

    /// Set the given bit in the bit pattern.
    ///
    /// See [BitSet::set].
    fn set(&mut self, index: usize);

    /// Clear the given bit in the bit pattern.
    ///
    /// See [BitSet::clear].
    fn clear(&mut self, index: usize);

    /// Construct an iterator over a bit pattern.
    ///
    /// See [BitSet::iter].
    fn iter(self) -> Self::Iter;
}

macro_rules! impl_num_bits {
    ($ty:ty) => {
        impl Number for $ty {
            fn trailing_zeros(self) -> usize {
                <Self>::trailing_zeros(self) as usize
            }

            fn is_zero(self) -> bool {
                self == 0
            }
        }

        impl Bits for $ty {
            type Iter = Iter<Self>;

            const EMPTY: Self = 0;
            const FULL: Self = !0;

            #[inline]
            fn test(&self, index: usize) -> bool {
                (*self & (1 << index)) != 0
            }

            #[inline]
            fn set(&mut self, index: usize) {
                *self |= 1 << index;
            }

            #[inline]
            fn clear(&mut self, index: usize) {
                *self &= !(1 << index);
            }

            #[inline]
            fn iter(self) -> Self::Iter {
                Iter { bits: self }
            }
        }
    };
}

impl_num_bits!(u128);
impl_num_bits!(u64);
impl_num_bits!(u32);
impl_num_bits!(u16);
impl_num_bits!(u8);

impl<T, const N: usize> Bits for [T; N]
where
    T: Bits + Number,
{
    type Iter = ArrayIter<T, N>;

    const EMPTY: Self = [T::EMPTY; N];
    const FULL: Self = [T::FULL; N];

    fn test(&self, index: usize) -> bool {
        if let Some(bits) = self.get(index / T::BITS) {
            return bits.test(index % T::BITS);
        }

        false
    }

    fn set(&mut self, index: usize) {
        if let Some(bits) = self.get_mut(index / T::BITS) {
            bits.set(index % T::BITS);
        }
    }

    fn clear(&mut self, index: usize) {
        if let Some(bits) = self.get_mut(index / T::BITS) {
            bits.clear(index % T::BITS);
        }
    }

    fn iter(self) -> Self::Iter {
        ArrayIter { bits: self, o: 0 }
    }
}

impl<T> Mask for BitSet<T>
where
    T: Bits,
{
    type Iter = T::Iter;

    #[inline]
    fn test(&self, index: usize) -> bool {
        <BitSet<T>>::test(self, index)
    }

    #[inline]
    fn iter(&self) -> Self::Iter {
        <BitSet<T>>::iter(self)
    }
}
