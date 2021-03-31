//! A library for working with small and cheap bit sets and masks
//!
//! Masks keep track of usize indexes which are set through
//! [testing][Mask::test]. This allows for masking indexes in certain
//! operations. Like if you want to mask which channels in an audio buffer is in
//! use or not.
//!
//! # Examples
//!
//! ```rust
//! fn test<M>(mask: M) where M: bittle::Mask {
//!     assert!(!mask.test(0));
//!     assert!(mask.test(1));
//! }
//!
//! let mut set = bittle::BitSet::<u16>::empty();
//! set.set(1);
//!
//! test(&set);
//!
//! set.clear(1);
//!
//! assert_eq!(std::mem::size_of_val(&set), std::mem::size_of::<u16>());
//! ```

#[macro_use]
mod macros;

mod mask;
pub use self::mask::Mask;

mod bit_set;
pub use self::bit_set::BitSet;

/// Construct the special mask where every index is set.
///
/// # Examples
///
/// ```rust
/// use bittle::Mask;
///
/// let n = bittle::all();
///
/// assert!(n.test(0));
/// assert!(n.test(usize::MAX));
/// ```
pub fn all() -> self::mask::All {
    self::mask::all::All::default()
}

/// Construct the special mask where no index is set.
///
/// # Examples
///
/// ```rust
/// use bittle::Mask;
///
/// let n = bittle::none();
///
/// assert!(!n.test(0));
/// assert!(!n.test(usize::MAX));
/// ```
pub fn none() -> self::mask::None {
    self::mask::none::None::default()
}
