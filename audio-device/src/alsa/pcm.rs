use crate::alsa::{
    ChannelArea, Configurator, Error, HardwareParameters, HardwareParametersMut, Result, Sample,
    SoftwareParameters, SoftwareParametersMut, Stream, Writer,
};
use crate::libc as c;
use crate::unix::poll::{PollFd, PollFlags};
use alsa_sys as alsa;
use std::ffi::CStr;
use std::mem;
use std::ptr;

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

    /// Construct a simple stream [Configurator].
    ///
    /// It will be initialized with a set of default parameters which are
    /// usually suitable for simple playback or recording for the given sample
    /// type `T`.
    ///
    /// See [Configurator].
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let config = pcm.configure::<i16>().install()?;
    /// # Ok(()) }
    /// ```
    pub fn configure<T>(&mut self) -> Configurator<'_, T>
    where
        T: Sample,
    {
        Configurator::new(self)
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
                stream as c::c_uint,
                flags
            ))?;

            Ok(Self {
                handle: ptr::NonNull::new_unchecked(handle.assume_init()),
            })
        }
    }

    /// Start a PCM.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// pcm.start()?;
    /// # Ok(()) }
    /// ```
    pub fn start(&mut self) -> Result<()> {
        unsafe {
            errno!(alsa::snd_pcm_start(self.handle.as_mut()))?;
            Ok(())
        }
    }

    /// Pause a PCM.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// pcm.pause()?;
    /// # Ok(()) }
    /// ```
    pub fn pause(&mut self) -> Result<()> {
        unsafe {
            errno!(alsa::snd_pcm_pause(self.handle.as_mut(), 1))?;
            Ok(())
        }
    }

    /// Resume a PCM.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// pcm.resume()?;
    /// # Ok(()) }
    /// ```
    pub fn resume(&mut self) -> Result<()> {
        unsafe {
            errno!(alsa::snd_pcm_pause(self.handle.as_mut(), 0))?;
            Ok(())
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

    /// Write unchecked interleaved frames to a PCM.
    ///
    /// Note: that the `len` must be the number of frames in the `buf` which
    /// *does not* account for the number of channels. So if `len` is 100, and
    /// the number of configured channels is 2, the `buf` must contain **at
    /// least** 200 bytes.
    ///
    /// See [HardwareParameters::channels].
    pub unsafe fn write_interleaved_unchecked(
        &mut self,
        buf: *const c::c_void,
        len: c::c_ulong,
    ) -> Result<c::c_long> {
        let written = errno!(alsa::snd_pcm_writei(self.handle.as_mut(), buf, len))?;
        Ok(written)
    }

    /// Construct a checked safe writer with the given number of channels and
    /// the specified sample type.
    ///
    /// This will error if the type `T` is not appropriate for this device, or
    /// if the number of channels does not match the number of configured
    /// channels.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let config = pcm.configure::<i16>().install()?;
    ///
    /// let mut writer = pcm.writer::<i16>()?;
    /// // use writer with the resulting config.
    /// # Ok(()) }
    /// ```
    pub fn writer<T>(&mut self) -> Result<Writer<'_, T>>
    where
        T: Sample,
    {
        let hw = self.hardware_parameters()?;
        let channels = hw.channels()? as usize;

        // NB: here we check that `T` is appropriate for the current format.
        let format = hw.format()?;

        if !T::test(format) {
            return Err(Error::FormatMismatch {
                ty: T::describe(),
                format,
            });
        }

        unsafe { Ok(Writer::new(self, channels)) }
    }

    /// Return number of frames ready to be read (capture) / written (playback).
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
    /// let avail = pcm.available_update()?;
    /// dbg!(avail);
    /// # Ok(()) }
    /// ```
    pub fn available_update(&mut self) -> Result<usize> {
        unsafe { Ok(errno!(alsa::snd_pcm_avail_update(self.handle.as_mut()))? as usize) }
    }

    // Application request to access a portion of direct (mmap) area.
    pub fn mmap_begin(&mut self, mut frames: c::c_ulong) -> Result<ChannelArea<'_>> {
        unsafe {
            let mut area = mem::MaybeUninit::uninit();
            let mut offset = mem::MaybeUninit::uninit();
            errno!(alsa::snd_pcm_mmap_begin(
                self.handle.as_mut(),
                area.as_mut_ptr(),
                offset.as_mut_ptr(),
                &mut frames
            ))?;
            let area = area.assume_init();
            let offset = offset.assume_init();

            Ok(ChannelArea {
                pcm: &mut self.handle,
                area,
                offset,
                frames,
            })
        }
    }
}

impl Drop for Pcm {
    fn drop(&mut self) {
        unsafe { alsa::snd_pcm_close(self.handle.as_ptr()) };
    }
}
