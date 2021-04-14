use alsa_sys as alsa;
use std::fmt;

macro_rules! decl_enum {
    (
        #[repr($ty:ident)]
        $vis:vis enum $name:ident {
            $(
                $(#[$m:meta])*
                $a:ident = $b:ident
            ),* $(,)?
        }
    ) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[non_exhaustive]
        #[repr($ty)]
        $vis enum $name {
            $(
                $(#[$m])*
                $a = alsa::$b,
            )*
        }

        impl $name {
            /// Parse the given enum from a value.
            $vis fn from_value(value: $ty) -> Option<Self> {
                Some(match value {
                    $(alsa::$b => Self::$a,)*
                    _ => return None,
                })
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let id = match self {
                    $(Self::$a => stringify!($a),)*
                };

                f.write_str(id)
            }
        }
    }
}

decl_enum! {
    #[repr(i32)]
    pub enum Format {
        Unknown = SND_PCM_FORMAT_UNKNOWN,
        S8 = SND_PCM_FORMAT_S8,
        U8 = SND_PCM_FORMAT_U8,
        S16LE = SND_PCM_FORMAT_S16_LE,
        S16BE = SND_PCM_FORMAT_S16_BE,
        U16LE = SND_PCM_FORMAT_U16_LE,
        U16BE = SND_PCM_FORMAT_U16_BE,
        S24LE = SND_PCM_FORMAT_S24_LE,
        S24BE = SND_PCM_FORMAT_S24_BE,
        U24LE = SND_PCM_FORMAT_U24_LE,
        U24BE = SND_PCM_FORMAT_U24_BE,
        S32LE = SND_PCM_FORMAT_S32_LE,
        S32BE = SND_PCM_FORMAT_S32_BE,
        U32LE = SND_PCM_FORMAT_U32_LE,
        U32BE = SND_PCM_FORMAT_U32_BE,
        FloatLE = SND_PCM_FORMAT_FLOAT_LE,
        FloatBE = SND_PCM_FORMAT_FLOAT_BE,
        Float64LE = SND_PCM_FORMAT_FLOAT64_LE,
        Float64BE = SND_PCM_FORMAT_FLOAT64_BE,
        IEC958SubframeLE = SND_PCM_FORMAT_IEC958_SUBFRAME_LE,
        IEC958SubframeBE = SND_PCM_FORMAT_IEC958_SUBFRAME_BE,
        MuLaw = SND_PCM_FORMAT_MU_LAW,
        ALaw = SND_PCM_FORMAT_A_LAW,
        ImaAdPCM = SND_PCM_FORMAT_IMA_ADPCM,
        MPEG = SND_PCM_FORMAT_MPEG,
        GSM = SND_PCM_FORMAT_GSM,
        Special = SND_PCM_FORMAT_SPECIAL,
        S243LE = SND_PCM_FORMAT_S24_3LE,
        S243BE = SND_PCM_FORMAT_S24_3BE,
        U243LE = SND_PCM_FORMAT_U24_3LE,
        U243BE = SND_PCM_FORMAT_U24_3BE,
        S203LE = SND_PCM_FORMAT_S20_3LE,
        S203BE = SND_PCM_FORMAT_S20_3BE,
        U203LE = SND_PCM_FORMAT_U20_3LE,
        U203BE = SND_PCM_FORMAT_U20_3BE,
        S183LE = SND_PCM_FORMAT_S18_3LE,
        S183BE = SND_PCM_FORMAT_S18_3BE,
        U183LE = SND_PCM_FORMAT_U18_3LE,
        U183BE = SND_PCM_FORMAT_U18_3BE,
        G72324 = SND_PCM_FORMAT_G723_24,
        G723241B = SND_PCM_FORMAT_G723_24_1B,
        G72340 = SND_PCM_FORMAT_G723_40,
        G723401B = SND_PCM_FORMAT_G723_40_1B,
        DSDU8 = SND_PCM_FORMAT_DSD_U8,
        DSDU16LE = SND_PCM_FORMAT_DSD_U16_LE,
        DSDU32LE = SND_PCM_FORMAT_DSD_U32_LE,
        DSDU16BE = SND_PCM_FORMAT_DSD_U16_BE,
        DSDU32BE = SND_PCM_FORMAT_DSD_U32_BE,
    }
}

decl_enum! {
    #[repr(u32)]
    pub enum Access {
        /// mmap access with simple interleaved channels
        MmapInterleaved = SND_PCM_ACCESS_MMAP_INTERLEAVED,
        /// mmap access with simple non interleaved channels
        MmapNoninterleaved = SND_PCM_ACCESS_MMAP_NONINTERLEAVED,
        /// mmap access with complex placement
        MmapComplex = SND_PCM_ACCESS_MMAP_COMPLEX,
        /// snd_pcm_readi/snd_pcm_writei access
        MmapReadWriteInterleaved = SND_PCM_ACCESS_RW_INTERLEAVED,
        /// snd_pcm_readn/snd_pcm_writen access
        MmapReadWriteNoninterleaved = SND_PCM_ACCESS_RW_NONINTERLEAVED,
    }
}

decl_enum! {
    #[repr(u32)]
    pub enum Timestamp {
        /// No timestamp.
        None = SND_PCM_TSTAMP_NONE,
        //// Update timestamp at every hardware position update.
        Enable = SND_PCM_TSTAMP_ENABLE,
    }
}

decl_enum! {
    #[repr(u32)]
    pub enum TimestampType {
        /// gettimeofday equivalent
        GetTimeOfDay = SND_PCM_TSTAMP_TYPE_GETTIMEOFDAY,
        /// posix_clock_monotonic equivalent
        Monotonic = SND_PCM_TSTAMP_TYPE_MONOTONIC,
        /// monotonic_raw (no NTP)
        MonotonicRaw = SND_PCM_TSTAMP_TYPE_MONOTONIC_RAW,
    }
}
