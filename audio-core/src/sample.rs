/// A sample that can be stored in an audio buffer. Types implementing this are
/// known as being sample apt.
///
/// Sample apt types have the following gurantees:
///
/// * The type does not need to be dropped (by implementing [Copy]).
/// * The type can safely be initialized with the all-zeros bit pattern.
///
/// # Safety
///
/// Implementor must make sure that a bit-pattern of all-zeros is a legal
/// bit-pattern for the implemented type.
pub unsafe trait Sample: Copy + Default {
    /// The zero pattern for the sample.
    const ZERO: Self;
}

/// The bit-pattern of all zeros is a legal bit-pattern for floats.
///
/// See for example:
/// <https://doc.rust-lang.org/std/primitive.f32.html#method.to_bits>.
///
/// Proof:
///
/// ```
/// use audio::Sample;
///
/// assert_eq!((f64::ZERO).to_bits(), 0u64);
/// ```
unsafe impl Sample for f32 {
    const ZERO: Self = 0.0;
}

/// The bit-pattern of all zeros is a legal bit-pattern for floats.
///
/// See for example:
/// <https://doc.rust-lang.org/std/primitive.f64.html#method.to_bits>.
///
/// Proof:
///
/// ```
/// use audio::Sample;
///
/// assert_eq!((f64::ZERO).to_bits(), 0u64);
/// ```
unsafe impl Sample for f64 {
    const ZERO: Self = 0.0;
}

// Helper macro to implement [Sample] for integer types.
macro_rules! impl_int {
    ($ty:ty) => {
        unsafe impl Sample for $ty {
            const ZERO: Self = 0;
        }
    };
}

// Note: trivial integer implementations.
impl_int!(u8);
impl_int!(u16);
impl_int!(u32);
impl_int!(u64);
impl_int!(u128);
impl_int!(i8);
impl_int!(i16);
impl_int!(i32);
impl_int!(i64);
impl_int!(i128);
impl_int!(usize);
impl_int!(isize);

// Helper macro to implement [Sample] for byte arrays.
macro_rules! impl_bytes {
    ($bytes:expr) => {
        unsafe impl Sample for [u8; $bytes] {
            const ZERO: Self = [0; $bytes];
        }
    };
}

// Implement for byte arrays of common lengths
impl_bytes!(2);
impl_bytes!(3);
impl_bytes!(4);
impl_bytes!(8);
impl_bytes!(16);
