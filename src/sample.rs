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
/// ```rust
/// use rotary::Sample as _;
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
/// ```rust
/// use rotary::Sample as _;
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

/// Trait used for translating one sample to another.
pub trait Translate<T> {
    /// Translate one kind of buffer to another.
    fn translate(value: T) -> Self;
}

macro_rules! translate_signed_to_float {
    ($from:ident, $to:ident) => {
        impl Translate<$from> for $to {
            fn translate(value: $from) -> Self {
                if value < 0 {
                    value as $to / -(::std::$from::MIN as $to)
                } else {
                    value as $to / ::std::$from::MAX as $to
                }
            }
        }
    };
}

macro_rules! translate_float_to_signed {
    ($from:ident, $to:ident) => {
        impl Translate<$from> for $to {
            fn translate(value: $from) -> Self {
                if value >= 0.0 {
                    (value * ::std::$to::MAX as $from) as $to
                } else {
                    (-value * ::std::$to::MIN as $from) as $to
                }
            }
        }
    };
}

translate_signed_to_float!(i16, f32);
translate_signed_to_float!(i16, f64);

translate_float_to_signed!(f32, i16);
translate_float_to_signed!(f64, i16);
