#[cfg(test)]
mod tests;

/// Trait used for translating one sample type to another.
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
pub trait Translate<T> {
    /// Translate one kind of buffer to another.
    fn translate(value: T) -> Self;
}

macro_rules! identity {
    ($ty:ty) => {
        impl Translate<$ty> for $ty {
            fn translate(value: $ty) -> Self {
                value
            }
        }
    };
}

macro_rules! int_to_float {
    ($signed:ident, $unsigned:ident, $float:ident) => {
        impl Translate<$signed> for $float {
            #[inline]
            fn translate(value: $signed) -> Self {
                if value < 0 {
                    (value as $float / -($signed::MIN as $float))
                } else {
                    (value as $float / $signed::MAX as $float)
                }
            }
        }

        impl Translate<$float> for $signed {
            #[inline]
            fn translate(value: $float) -> Self {
                if value >= 0.0 {
                    (value * $signed::MAX as $float) as $signed
                } else {
                    (-value * $signed::MIN as $float) as $signed
                }
            }
        }

        #[cfg(feature = "std")]
        impl Translate<$float> for $unsigned {
            #[inline]
            fn translate(value: $float) -> Self {
                let value = value.clamp(-1.0, 1.0);

                (((value + 1.0) * 0.5) * $unsigned::MAX as $float).round() as $unsigned
            }
        }

        impl Translate<$unsigned> for $float {
            #[inline]
            fn translate(value: $unsigned) -> Self {
                // Note: less conversion loss the closer we stay to 0.
                $float::translate($signed::translate(value))
            }
        }
    };
}

macro_rules! signed_to_unsigned {
    ($signed:ty, $unsigned:ty) => {
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
    };
}

identity!(f32);
identity!(f64);
identity!(i16);
identity!(u16);
identity!(u8);

int_to_float!(i16, u16, f32);
int_to_float!(i16, u16, f64);

int_to_float!(i8, u8, f32);
int_to_float!(i8, u8, f64);

signed_to_unsigned!(i16, u16);
signed_to_unsigned!(i8, u8);

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
