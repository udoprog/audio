use crate::alsa::{HardwareParametersAny, HardwareParametersCurrent, Result};
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

    /// Open all available hardware parameters for the current handle.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let hw = pcm.hardware_parameters_any()?;
    /// # Ok(()) }
    /// ```
    pub fn hardware_parameters_any(&mut self) -> Result<HardwareParametersAny<'_>> {
        unsafe { HardwareParametersAny::new(&mut self.handle) }
    }

    /// Open current hardware parameters for the current handle.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let hw = pcm.hardware_parameters_current()?;
    /// dbg!(hw.rate()?);
    /// # Ok(()) }
    /// ```
    pub fn hardware_parameters_current(&mut self) -> Result<HardwareParametersCurrent> {
        unsafe { HardwareParametersCurrent::new(&mut self.handle) }
    }

    /// Get count of poll descriptors for PCM handle.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let count = pcm.poll_descriptors_count();
    /// dbg!(count);
    /// # Ok(()) }
    /// ```
    pub fn poll_descriptors_count(&mut self) -> usize {
        unsafe { alsa::snd_pcm_poll_descriptors_count(self.handle.as_mut()) as usize }
    }

    /// Get poll descriptors.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let count = pcm.poll_descriptors_count();
    /// let mut fds = vec![libc::pollfd { fd: 0, events: 0, revents: 0 }; count];
    /// let filled = pcm.poll_descriptors(&mut fds[..])?;
    /// # Ok(()) }
    /// ```
    pub fn poll_descriptors(&mut self, fds: &mut [libc::pollfd]) -> Result<usize> {
        unsafe {
            let result = errno!(alsa::snd_pcm_poll_descriptors(
                self.handle.as_mut(),
                fds.as_mut_ptr(),
                fds.len() as libc::c_uint
            ))?;
            Ok(result as usize)
        }
    }
}

impl Drop for Pcm {
    fn drop(&mut self) {
        unsafe { alsa::snd_pcm_close(self.handle.as_ptr()) };
    }
}
