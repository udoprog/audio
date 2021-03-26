/// A sample that can be stored in an audio buffer.
///
/// `Copy` guarantees that the sample cannot have a destructor.
///
/// # Safety
///
/// Implementor must make sure that a bit-pattern of all-zeros is a legal
/// bit-pattern for the implemented type.
pub unsafe trait Sample: Copy + Default {}

/// The bit-pattern of all zeros is a legal bit-pattern for floats.
///
/// See for example:
/// <https://doc.rust-lang.org/std/primitive.f32.html#method.to_bits>.
///
/// Proof:
///
/// ```rust
/// assert_eq!((0.0f32).to_bits(), 0u32);
/// assert_eq!((0.0f64).to_bits(), 0u64);
/// ```
unsafe impl Sample for f32 {}
unsafe impl Sample for f64 {}
