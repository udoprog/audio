use crate::alsa::Format;

/// Trait used to designate types which are sample-appropriate for
/// [Pcm][super::Pcm].
///
/// # Safety
///
/// This trait is unsafe to implement, because an incorrectly implemented format
/// test might have safety implications.
pub unsafe trait Sample {
    /// The default format to use for this sample.
    const DEFAULT_FORMAT: Format;

    /// Test if the given format is appropriate for this sample type.
    fn test(format: Format) -> bool;

    /// A static description of the sample type.
    fn describe() -> &'static str;
}

macro_rules! implement {
    ($ty:ty, $le:ident, $be:ident) => {
        unsafe impl Sample for $ty {
            #[cfg(target_endian = "little")]
            const DEFAULT_FORMAT: Format = Format::$le;
            #[cfg(target_endian = "big")]
            const DEFAULT_FORMAT: Format = Format::$be;

            fn test(format: Format) -> bool {
                match format {
                    #[cfg(target_endian = "little")]
                    Format::$le => true,
                    #[cfg(target_endian = "big")]
                    Format::$be => true,
                    _ => false,
                }
            }

            #[cfg(target_endian = "little")]
            fn describe() -> &'static str {
                concat!(stringify!($ty), " (little endian)")
            }

            #[cfg(target_endian = "big")]
            fn describe() -> &'static str {
                concat!(stringify!($ty), " (big endian)")
            }
        }
    };
}

unsafe impl Sample for u8 {
    const DEFAULT_FORMAT: Format = Format::U8;

    fn test(format: Format) -> bool {
        matches!(format, Format::U8)
    }

    #[cfg(target_endian = "little")]
    fn describe() -> &'static str {
        "u8 (little endian)"
    }

    #[cfg(target_endian = "big")]
    fn describe() -> &'static str {
        "u8 (big endian)"
    }
}

unsafe impl Sample for i8 {
    const DEFAULT_FORMAT: Format = Format::S8;

    fn test(format: Format) -> bool {
        matches!(format, Format::S8)
    }

    #[cfg(target_endian = "little")]
    fn describe() -> &'static str {
        "i8 (little endian)"
    }

    #[cfg(target_endian = "big")]
    fn describe() -> &'static str {
        "i8 (big endian)"
    }
}

implement!(i16, S16LE, S16BE);
implement!(u16, U16LE, U16BE);
implement!(i32, S32LE, S32BE);
implement!(u32, U32LE, U32BE);
implement!(f32, FloatLE, FloatBE);
implement!(f64, Float64LE, Float64BE);
