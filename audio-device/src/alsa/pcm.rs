use crate::alsa::{HardwareParameters, Result};
use alsa_sys as alsa;
use std::ffi::CStr;
use std::mem;
use std::ptr;

#[derive(Debug, Clone, Copy)]
pub enum Stream {
    /// A capture stream. Corresponds to `SND_PCM_STREAM_CAPTURE`.
    Capture,
    /// A playback stream. Corresponds to `SND_PCM_STREAM_PLAYBACK`.
    Playback,
}

impl Stream {
    fn into_flag(self) -> u32 {
        match self {
            Stream::Capture => alsa::SND_PCM_STREAM_CAPTURE,
            Stream::Playback => alsa::SND_PCM_STREAM_CAPTURE,
        }
    }
}

/// An opened PCM device.
pub struct Pcm {
    pub(super) handle: ptr::NonNull<alsa::snd_pcm_t>,
}

impl Pcm {
    /// Open the given pcm device identified by name.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    /// use std::ffi::CStr;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let name = CStr::from_bytes_with_nul(b"hw:0\0")?;
    ///
    /// let pcm = alsa::Pcm::open(name, alsa::Stream::Playback)?;
    /// # Ok(()) }
    /// ```
    pub fn open(name: &CStr, stream: Stream) -> Result<Self> {
        Self::open_inner(name, stream, 0)
    }

    /// Open the default pcm device.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    /// use std::ffi::CStr;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// # Ok(()) }
    /// ```
    pub fn open_default(stream: Stream) -> Result<Self> {
        static DEFAULT: &[u8] = b"default\0";
        Self::open(
            unsafe { CStr::from_bytes_with_nul_unchecked(DEFAULT) },
            stream,
        )
    }

    /// Open the given pcm device identified by name in a nonblocking manner.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    /// use std::ffi::CStr;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let name = CStr::from_bytes_with_nul(b"hw:0\0")?;
    ///
    /// let pcm = alsa::Pcm::open_nonblocking(name, alsa::Stream::Playback)?;
    /// # Ok(()) }
    /// ```
    pub fn open_nonblocking(name: &CStr, stream: Stream) -> Result<Self> {
        Self::open_inner(name, stream, alsa::SND_PCM_NONBLOCK)
    }

    fn open_inner(name: &CStr, stream: Stream, flags: i32) -> Result<Self> {
        unsafe {
            let mut handle = mem::MaybeUninit::uninit();

            errno!(alsa::snd_pcm_open(
                handle.as_mut_ptr(),
                name.as_ptr(),
                stream.into_flag(),
                flags
            ))?;

            Ok(Self {
                handle: ptr::NonNull::new_unchecked(handle.assume_init()),
            })
        }
    }

    /// Open hardware parameters for the current handle.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    /// use std::ffi::CStr;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let hw = pcm.hardware_parameters_any()?;
    /// # Ok(()) }
    /// ```
    pub fn hardware_parameters_any(&self) -> Result<HardwareParameters> {
        unsafe { HardwareParameters::any(&self.handle) }
    }
}

impl Drop for Pcm {
    fn drop(&mut self) {
        unsafe { alsa::snd_pcm_close(self.handle.as_ptr()) };
    }
}
