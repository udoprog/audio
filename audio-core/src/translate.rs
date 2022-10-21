//! Utility traits for dealing with sample translations.
//!
//! Primitive samples are encoded with PCM which have a midpoint of no amplitude
//! (i.e. silence). The possible primitives are as follows:
//!
//! * Unsigned samples have a span from 0 as its *highest negative* amplitude to
//!   maximum as its *highest positive* amplitude. The midpoint is defined as
//!   its `(max + 1) / 2` such as `0x8000` for `u16` or `0x80000000` for `u32`.
//! * Signed samples have a midpoint at 0 and utilises the full range of the
//!   type where its minimum is the *highest negative* amplitude and its maximum
//!   is its *highest positive* amplitude.
//! * Float samples have a midpoint at `0.0` and utilises the range `-1.0` to
//!   `1.0` (inclusive).
//!
//! These rules are applied to the following *native* Rust types:
//!
//! * `u8`, `u16`, `u32`, and `u64` for unsigned PCM 8 to 64 bit audio
//!   modulation.
//! * `i8`, `i16`, `i32`, and `i64` for signed PCM 8 to 64 bit audio modulation.
//! * `f32` and `f64` for 32 and 64 bit PCM floating-point audio modulation.
//!
//! The primary traits that govern how something is translated are the
//! [Translate] and [TryTranslate]. The first deals with non-fallible
//! translations where conversion loss is expected (as with float-integer
//! translations). [TryTranslate] deals with translations where an unexpected
//! loss in precision would otherwise occur.
//!
//! See the documentation for each trait for more information.

use core::convert::Infallible;

#[cfg(test)]
mod tests;

/// Trait used for translating one sample type to another.
///
/// This performs infallible translations where any loss in precision is
/// *expected* and is not supported between types which cannot be universally
/// translated in this manner such as translations from a higher to a lower
/// precision format.
///
/// # Examples
///
/// ```
/// use audio::Translate;
///
/// assert_eq!(i16::translate(-1.0f32), i16::MIN);
/// assert_eq!(i16::translate(0.0f32), 0);
///
/// assert_eq!(u16::translate(-1.0f32), u16::MIN);
/// assert_eq!(u16::translate(0.0f32), 32768);
/// ```
pub trait Translate<T>: Sized {
    /// Translate one kind of buffer to another.
    fn translate(value: T) -> Self;
}

/// Trait for performing checked translations, where it's checked if a
/// translation would result in loss of precision.
///
/// This will fail if we try to perform a translation between two integer types
/// which are not exactly equivalent.
///
/// # Examples
///
/// ```
/// use audio::translate::{TryTranslate, IntTranslationError};
///
/// assert_eq!(i16::try_translate(-1.0f32), Ok(i16::MIN));
/// assert_eq!(i16::try_translate(i32::MIN), Ok(i16::MIN));
/// assert_eq!(i16::try_translate(0x70000000i32), Ok(0x7000i16));
/// assert!(matches!(i16::try_translate(0x70000001i32), Err(IntTranslationError { .. })));
/// ```
pub trait TryTranslate<T>: Sized {
    /// Error kind raised from the translation.
    type Err;

    /// Perform a conversion.
    fn try_translate(value: T) -> Result<Self, Self::Err>;
}

impl<T, U> TryTranslate<T> for U
where
    U: Translate<T>,
{
    type Err = Infallible;

    #[inline]
    fn try_translate(value: T) -> Result<Self, Self::Err> {
        Ok(U::translate(value))
    }
}

/// Unable to translate an integer due to loss of precision.
#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct IntTranslationError;

macro_rules! identity {
    ($ty:ty) => {
        impl Translate<$ty> for $ty {
            #[inline]
            fn translate(value: $ty) -> Self {
                value
            }
        }
    };
}

macro_rules! conversions {
    (
        $signed:ty, $unsigned:ty,
        {$($float:ty),* $(,)?},
        {$([$lossless_unsigned:ty, $lossless_signed:ty, $lossless_shift:expr]),* $(,)?}
    ) => {
        identity!($signed);
        identity!($unsigned);

        impl Translate<$unsigned> for $signed {
            #[inline]
            fn translate(value: $unsigned) -> Self {
                (value as $signed).wrapping_sub(<$signed>::MIN)
            }
        }

        impl Translate<$signed> for $unsigned {
            #[inline]
            fn translate(value: $signed) -> Self {
                value.wrapping_add(<$signed>::MIN) as $unsigned
            }
        }

        $(
        impl Translate<$lossless_unsigned> for $unsigned {
            #[inline]
            fn translate(value: $lossless_unsigned) -> Self {
                (value as $unsigned).wrapping_shl($lossless_shift)
            }
        }

        impl Translate<$lossless_signed> for $unsigned {
            #[inline]
            fn translate(value: $lossless_signed) -> Self {
                <$unsigned>::translate(<$lossless_unsigned>::translate(value))
            }
        }

        impl Translate<$lossless_signed> for $signed {
            #[inline]
            fn translate(value: $lossless_signed) -> Self {
                (value as $signed).wrapping_shl($lossless_shift)
            }
        }

        impl Translate<$lossless_unsigned> for $signed {
            #[inline]
            fn translate(value: $lossless_unsigned) -> Self {
                <$signed>::translate(<$lossless_signed>::translate(value))
            }
        }

        impl TryTranslate<$unsigned> for $lossless_unsigned {
            type Err = IntTranslationError;

            #[inline]
            fn try_translate(value: $unsigned) -> Result<Self, Self::Err> {
                if value & ((1 << $lossless_shift) - 1) != 0 {
                    return Err(IntTranslationError);
                }

                Ok(value.wrapping_shr($lossless_shift) as $lossless_unsigned)
            }
        }

        impl TryTranslate<$signed> for $lossless_unsigned {
            type Err = IntTranslationError;

            #[inline]
            fn try_translate(value: $signed) -> Result<Self, Self::Err> {
                <$lossless_unsigned>::try_translate(<$unsigned>::translate(value))
            }
        }

        impl TryTranslate<$signed> for $lossless_signed {
            type Err = IntTranslationError;

            #[inline]
            fn try_translate(value: $signed) -> Result<Self, Self::Err> {
                if value & ((1 << $lossless_shift) - 1) != 0 {
                    return Err(IntTranslationError);
                }

                Ok(value.wrapping_shr($lossless_shift) as $lossless_signed)
            }
        }

        impl TryTranslate<$unsigned> for $lossless_signed {
            type Err = IntTranslationError;

            #[inline]
            fn try_translate(value: $unsigned) -> Result<Self, Self::Err> {
                <$lossless_signed>::try_translate(<$signed>::translate(value))
            }
        }
        )*

        $(
        impl Translate<$signed> for $float {
            #[inline]
            fn translate(value: $signed) -> Self {
                // Needs special care to avoid distortion at the cost of not
                // covering the whole range.
                // See: https://github.com/udoprog/audio/issues/7
                -(value as $float) / (<$signed>::MIN as $float)
            }
        }

        impl Translate<$float> for $signed {
            #[inline]
            fn translate(value: $float) -> Self {
                // Needs special care to avoid distortion at the cost of not
                // covering the whole range.
                // See: https://github.com/udoprog/audio/issues/7
                -(value * <$signed>::MIN as $float) as $signed
            }
        }

        #[cfg(feature = "std")]
        impl Translate<$float> for $unsigned {
            #[inline]
            fn translate(value: $float) -> Self {
                // Go through signed to get the same float conversion.
                <$unsigned>::translate(<$signed>::translate(value))
            }
        }

        impl Translate<$unsigned> for $float {
            #[inline]
            fn translate(value: $unsigned) -> Self {
                // Go through signed to get the same float conversion.
                <$float>::translate(<$signed>::translate(value))
            }
        }
        )*
    };
}

identity!(f32);
identity!(f64);

impl Translate<f32> for f64 {
    #[inline]
    fn translate(value: f32) -> Self {
        value as f64
    }
}

impl Translate<f64> for f32 {
    #[inline]
    fn translate(value: f64) -> Self {
        value as f32
    }
}

conversions!(i64, u64, {f32, f64}, {[u8, i8, 56], [u16, i16, 48], [u32, i32, 32]});
conversions!(i32, u32, {f32, f64}, {[u8, i8, 24], [u16, i16, 16]});
conversions!(i16, u16, {f32, f64}, {[u8, i8, 8]});
conversions!(i8, u8, {f32, f64}, {});
