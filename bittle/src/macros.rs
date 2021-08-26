/// Construct a bit set with specific values set.
///
/// # Examples
///
/// ```
/// let mask: bittle::BitSet<u128> = bittle::bit_set![0, 1, 3];
///
/// assert!(mask.test(0));
/// assert!(mask.test(1));
/// assert!(!mask.test(2));
/// assert!(mask.test(3));
/// ```
#[macro_export]
macro_rules! bit_set {
    ($($set:expr),* $(,)?) => {
        $crate::BitSet::from_array([$($set,)*])
    };
}
