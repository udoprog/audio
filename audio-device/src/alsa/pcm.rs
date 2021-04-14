use crate::alsa::{
    HardwareParameters, HardwareParametersMut, Result, SoftwareParameters, SoftwareParametersMut,
};
use crate::libc as c;
use crate::unix::poll::{PollFd, PollFlags};
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
    /// let mut hw = pcm.hardware_parameters_any()?;
    /// hw.set_rate_last()?;
    /// hw.install()?;
    /// # Ok(()) }
    /// ```
    pub fn hardware_parameters_any(&mut self) -> Result<HardwareParametersMut<'_>> {
        unsafe { HardwareParametersMut::any(&mut self.handle) }
    }

    /// Open current hardware parameters for the current handle for mutable access.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    ///
    /// let mut hw = pcm.hardware_parameters_mut()?;
    /// let actual_rate = hw.set_rate(44100, alsa::Direction::Nearest)?;
    /// hw.install()?;
    ///
    /// dbg!(actual_rate);
    /// # Ok(()) }
    /// ```
    pub fn hardware_parameters_mut(&mut self) -> Result<HardwareParametersMut<'_>> {
        unsafe { HardwareParametersMut::current(&mut self.handle) }
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
    /// let sw = pcm.hardware_parameters()?;
    /// dbg!(sw.rate()?);
    /// # Ok(()) }
    /// ```
    pub fn hardware_parameters(&mut self) -> Result<HardwareParameters> {
        unsafe { HardwareParameters::current(&mut self.handle) }
    }

    /// Open current software parameters for the current handle.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let sw = pcm.software_parameters()?;
    ///
    /// dbg!(sw.boundary()?);
    /// # Ok(()) }
    /// ```
    pub fn software_parameters(&mut self) -> Result<SoftwareParameters> {
        unsafe { SoftwareParameters::new(&mut self.handle) }
    }

    /// Open current software parameters for the current handle for mutable access.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut sw = pcm.software_parameters_mut()?;
    ///
    /// sw.set_timestamp_mode(alsa::Timestamp::Enable)?;
    /// sw.install()?;
    /// # Ok(()) }
    /// ```
    pub fn software_parameters_mut(&mut self) -> Result<SoftwareParametersMut<'_>> {
        unsafe { SoftwareParametersMut::new(&mut self.handle) }
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
    /// This function fills the given poll descriptor structs for the specified
    /// PCM handle. The poll desctiptor array should have the size returned by
    /// [poll_descriptors_count()][Pcm::poll_descriptors_count()] function.
    ///
    /// The result is intended for direct use with the `poll()` syscall.
    ///
    /// For reading the returned events of poll descriptor after `poll()` system
    /// call, use ::snd_pcm_poll_descriptors_revents() function. The field
    /// values in pollfd structs may be bogus regarding the stream direction
    /// from the application perspective (`POLLIN` might not imply read
    /// direction and `POLLOUT` might not imply write), but the
    /// [poll_descriptors_revents()][Pcm::poll_descriptors_revents()] function
    /// does the right "demangling".
    ///
    /// You can use output from this function as arguments for the select()
    /// syscall, too. Do not forget to translate `POLLIN` and `POLLOUT` events
    /// to corresponding `FD_SET` arrays and demangle events using
    /// [poll_descriptors_revents()][Pcm::poll_descriptors_revents()].
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    /// use audio_device::unix::poll::PollFd;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    ///
    /// let mut fds = Vec::with_capacity(pcm.poll_descriptors_count());
    /// pcm.poll_descriptors(&mut fds)?;
    /// # Ok(()) }
    /// ```
    pub fn poll_descriptors(&mut self, fds: &mut Vec<PollFd>) -> Result<()> {
        unsafe {
            let count = self.poll_descriptors_count();

            if fds.capacity() < count {
                fds.reserve(count - fds.capacity());
            }

            let result = errno!(alsa::snd_pcm_poll_descriptors(
                self.handle.as_mut(),
                fds.as_mut_ptr() as *mut c::pollfd,
                fds.capacity() as c::c_uint
            ))?;

            let result = result as usize;

            assert!(result <= fds.capacity());
            fds.set_len(result);
            Ok(())
        }
    }

    /// Get returned events from poll descriptors.
    ///
    /// This function does "demangling" of the revents mask returned from the
    /// [poll()][crate::unix::poll::poll()] syscall to correct semantics
    /// ([PollFlags::POLLIN] = read, [PollFlags::POLLOUT] = write).
    ///
    /// Note: The null event also exists. Even if `poll()` or `select()` syscall
    /// returned that some events are waiting, this function might return empty
    /// set of events. In this case, application should do next event waiting
    /// using [poll()][crate::unix::poll::poll()] or `select()`.
    ///
    /// Note: Even if multiple poll descriptors are used (i.e. `fds.len() > 1`),
    /// this function returns only a single event.
    pub fn poll_descriptors_revents(&mut self, fds: &mut [PollFd]) -> Result<PollFlags> {
        unsafe {
            let mut revents = mem::MaybeUninit::uninit();
            errno!(alsa::snd_pcm_poll_descriptors_revents(
                self.handle.as_mut(),
                // NB: PollFd is `#[repr(transparent)]` around pollfd.
                fds.as_mut_ptr() as *mut c::pollfd,
                fds.len() as c::c_uint,
                revents.as_mut_ptr(),
            ))?;
            let revents = revents.assume_init();
            Ok(PollFlags::from_bits_truncate(revents as c::c_short))
        }
    }

    /// Write interleaved frames to a PCM.
    pub fn write_interleaved(&mut self, buffer: &[u8]) -> Result<usize> {
        unsafe {
            let written = errno!(alsa::snd_pcm_writei(
                self.handle.as_mut(),
                buffer.as_ptr() as *const _,
                buffer.len() as c::c_ulong
            ))?;

            Ok(written as usize)
        }
    }
}

impl Drop for Pcm {
    fn drop(&mut self) {
        unsafe { alsa::snd_pcm_close(self.handle.as_ptr()) };
    }
}
