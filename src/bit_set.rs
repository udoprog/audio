//! A fixed size bit set.

/// A fixed size bit set.
///
/// # Examples
///
/// ```rust
/// use rotary::Mask as _;
///
/// let mut set = rotary::BitSet::<u128>::empty();
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
/// use rotary::Mask as _;
///
/// let mut set = rotary::BitSet::<[u64; 16]>::empty();
///
/// assert!(!set.test(172));
/// set.set(172);
/// assert!(set.test(172));
/// set.clear(172);
/// assert!(!set.test(172));
/// ```
#[derive(Clone, Copy)]
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
    /// Construct a new bit set that is empty, where no element is set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let set = rotary::BitSet::<u128>::empty();
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
    /// let set = rotary::BitSet::<u128>::full();
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
    /// let mut set = rotary::BitSet::<u128>::full();
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
    /// let mut set = rotary::BitSet::<u128>::full();
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
    /// let mut set = rotary::BitSet::<u128>::full();
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
    /// let mut set = rotary::BitSet::<u128>::empty();
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
    /// use rotary::Mask as _;
    ///
    /// let mut set = rotary::BitSet::<[u32; 4]>::empty();
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
pub struct Iter<T>
where
    T: Bits + Num,
{
    bits: T,
}

impl<T> Iterator for Iter<T>
where
    T: Bits + Num,
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bits.is_empty() {
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
    T: Bits + Num,
{
    bits: [T; N],
    o: usize,
}

impl<T, const N: usize> Iterator for ArrayIter<T, N>
where
    T: Bits + Num,
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.o >= N {
                return None;
            }

            let bits = &mut self.bits[self.o];

            if bits.is_empty() {
                self.o += 1;
                continue;
            }

            let index = bits.trailing_zeros();
            bits.clear(index);
            return Some(self.o * T::BITS + index);
        }
    }
}

/// Basic numerical traits for the plumbing of a bit set.
pub trait Num: Copy {
    const BITS: usize;

    /// Trailing zeros.
    fn trailing_zeros(self) -> usize;

    /// Test if the bits are empty.
    fn is_empty(self) -> bool;
}

/// The trait for a bit set.
pub trait Bits: Copy {
    type Iter: Iterator<Item = usize>;

    /// Bit-pattern for an empty bit set.
    const EMPTY: Self;
    /// Bit-pattern for a full bit set.
    const FULL: Self;

    /// Test if the given bit is set.
    fn test(self, index: usize) -> bool;

    /// Set the given bit.
    fn set(&mut self, index: usize);

    /// Clear the given bit.
    fn clear(&mut self, index: usize);

    fn iter(self) -> Self::Iter;
}

macro_rules! impl_bits {
    ($ty:ty) => {
        impl Num for $ty {
            const BITS: usize = std::mem::size_of::<$ty>() * 8;

            fn trailing_zeros(self) -> usize {
                <$ty>::trailing_zeros(self) as usize
            }

            fn is_empty(self) -> bool {
                self == 0
            }
        }

        impl Bits for $ty {
            type Iter = Iter<$ty>;

            const EMPTY: Self = 0;
            const FULL: Self = !0;

            fn test(self, index: usize) -> bool {
                (self & (1 << index)) != 0
            }

            fn set(&mut self, index: usize) {
                *self |= 1 << index;
            }

            fn clear(&mut self, index: usize) {
                *self &= !(1 << index);
            }

            fn iter(self) -> Self::Iter {
                Iter { bits: self }
            }
        }
    };
}

impl_bits!(u128);
impl_bits!(u64);
impl_bits!(u32);

impl<T, const N: usize> Bits for [T; N]
where
    T: Bits + Num,
{
    type Iter = ArrayIter<T, N>;

    const EMPTY: Self = [T::EMPTY; N];
    const FULL: Self = [T::FULL; N];

    fn test(self, index: usize) -> bool {
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
