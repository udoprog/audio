use crate::alsa::Result;
use crate::libc as c;
use alsa_sys as alsa;
use std::fmt;

macro_rules! decl_enum {
    (
        $(#[doc = $doc:literal])*
        #[repr($ty:ident)]
        $vis:vis enum $name:ident {
            $(
                $(#[$m:meta])*
                $a:ident = $b:ident
            ),* $(,)?
        }
    ) => {
        $(#[doc = $doc])*
        #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[non_exhaustive]
        #[repr($ty)]
        $vis enum $name {
            $(
                $(#[$m])*
                #[allow(missing_docs)]
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

/// The direction in which updated hardware parameters is restricted unless the
/// exact value is available.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
pub enum Direction {
    /// Accept smaller values.
    Smaller = -1,
    /// Accept the nearest value.
    Nearest = 0,
    /// Accept greater values.
    Greater = 1,
}

impl Direction {
    pub(super) fn from_value(value: i32) -> Self {
        match value {
            -1 => Self::Smaller,
            0 => Self::Nearest,
            _ => Self::Greater,
        }
    }
}

decl_enum! {
    /// The state of the [Pcm][super::Pcm].
    #[repr(u32)]
    pub enum State {
        /// Open.
        Open = SND_PCM_STATE_OPEN,
        /// Setup installed.
        Setup = SND_PCM_STATE_SETUP,
        /// Ready to start.
        Prepare = SND_PCM_STATE_PREPARED,
        /// Running.
        Running = SND_PCM_STATE_RUNNING,
        /// Stopped: underrun (playback) or overrun (capture) detected.
        Xrun = SND_PCM_STATE_XRUN,
        /// Draining: running (playback) or stopped (capture).
        Draining = SND_PCM_STATE_DRAINING,
        /// Paused.
        Paused = SND_PCM_STATE_PAUSED,
        /// Hardware is suspended.
        Suspended = SND_PCM_STATE_SUSPENDED,
        /// Hardware is disconnected.
        Disconnected = SND_PCM_STATE_DISCONNECTED,
        /// Private - used internally in the library - do not use.
        Private1 = SND_PCM_STATE_PRIVATE1,
    }
}

decl_enum! {
    /// Defines the supported format of a stream.
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

impl Format {
    /// Return bits needed to store a PCM sample.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// assert_eq!(alsa::Format::U8.physical_width()?, 8);
    /// assert_eq!(alsa::Format::S16LE.physical_width()?, 16);
    /// assert_eq!(alsa::Format::S243LE.physical_width()?, 24);
    /// # Ok(()) }
    /// ```
    pub fn physical_width(self) -> Result<usize> {
        unsafe { Ok(errno!(alsa::snd_pcm_format_physical_width(self as c::c_int))? as usize) }
    }
}

decl_enum! {
    /// Defines the direction of a stream.
    ///
    /// See [Pcm::open][super::Pcm::open].
    #[repr(u32)]
    pub enum Stream {
        /// A capture stream. Corresponds to `SND_PCM_STREAM_CAPTURE`.
        Capture = SND_PCM_STREAM_CAPTURE,
        /// A playback stream. Corresponds to `SND_PCM_STREAM_PLAYBACK`.
        Playback = SND_PCM_STREAM_PLAYBACK,
    }
}

decl_enum! {
    /// Defines how the underlying device is accessed.
    #[repr(u32)]
    pub enum Access {
        /// MMAP access with simple interleaved channels
        MmapInterleaved = SND_PCM_ACCESS_MMAP_INTERLEAVED,
        /// MMAP access with simple non interleaved channels
        MmapNoninterleaved = SND_PCM_ACCESS_MMAP_NONINTERLEAVED,
        /// MMAP access with complex placement
        MmapComplex = SND_PCM_ACCESS_MMAP_COMPLEX,
        /// Interleaved read/write access
        ReadWriteInterleaved = SND_PCM_ACCESS_RW_INTERLEAVED,
        /// Sequential read/write access
        ReadWriteNoninterleaved = SND_PCM_ACCESS_RW_NONINTERLEAVED,
    }
}

decl_enum! {
    /// Defines if timestamps are enabled or not.
    #[repr(u32)]
    pub enum Timestamp {
        /// No timestamp.
        None = SND_PCM_TSTAMP_NONE,
        //// Update timestamp at every hardware position update.
        Enable = SND_PCM_TSTAMP_ENABLE,
    }
}

decl_enum! {
    /// Defines the type of timestamp that is available.
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
