use crate::alsa::{Error, Result, Timestamp, TimestampType};
use crate::libc as c;
use alsa_sys as alsa;
use std::mem;
use std::ops;
use std::ptr;

/// Collection of software parameters being configured for a [Pcm][super::Pcm]
/// handle.
///
/// See
/// [Pcm::software_parameters][super::Pcm::software_parameters].
pub struct SoftwareParameters {
    handle: ptr::NonNull<alsa::snd_pcm_sw_params_t>,
}

impl SoftwareParameters {
    /// Open current software parameters for the current device for reading.
    pub(super) unsafe fn new(pcm: &mut ptr::NonNull<alsa::snd_pcm_t>) -> Result<Self> {
        let mut handle = mem::MaybeUninit::uninit();

        errno!(alsa::snd_pcm_sw_params_malloc(handle.as_mut_ptr()))?;

        let mut handle = ptr::NonNull::new_unchecked(handle.assume_init());

        if let Err(e) = errno!(alsa::snd_pcm_sw_params_current(
            pcm.as_ptr(),
            handle.as_mut()
        )) {
            alsa::snd_pcm_sw_params_free(handle.as_mut());
            return Err(e);
        }

        Ok(SoftwareParameters { handle })
    }

    /// Copy one set of software parameters to another.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut a = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut b = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    ///
    /// let a = a.software_parameters()?;
    /// let mut b = b.software_parameters()?;
    ///
    /// b.copy(&a);
    /// # Ok(()) }
    /// ```
    pub fn copy(&mut self, other: &SoftwareParameters) {
        unsafe { alsa::snd_pcm_sw_params_copy(self.handle.as_mut(), other.handle.as_ptr()) };
    }

    /// Get boundary for ring pointers from a software configuration container.
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
    /// let boundary = sw.boundary()?;
    /// dbg!(boundary);
    /// # Ok(()) }
    /// ```
    pub fn boundary(&self) -> Result<c::c_ulong> {
        unsafe {
            let mut boundary = mem::MaybeUninit::uninit();
            errno!(alsa::snd_pcm_sw_params_get_boundary(
                self.handle.as_ptr(),
                boundary.as_mut_ptr()
            ))?;
            Ok(boundary.assume_init())
        }
    }

    /// Get timestamp mode from a software configuration container.
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
    /// let timestamp_mode = sw.timestamp_mode()?;
    /// dbg!(timestamp_mode);
    /// # Ok(()) }
    /// ```
    pub fn timestamp_mode(&self) -> Result<Timestamp> {
        unsafe {
            let mut timestamp_mode = mem::MaybeUninit::uninit();
            alsa::snd_pcm_sw_params_get_tstamp_mode(
                self.handle.as_ptr(),
                timestamp_mode.as_mut_ptr(),
            );
            let timestamp_mode = timestamp_mode.assume_init();
            let timestamp_mode = Timestamp::from_value(timestamp_mode)
                .ok_or_else(|| Error::BadTimestamp(timestamp_mode))?;
            Ok(timestamp_mode)
        }
    }

    /// Get timestamp type from a software configuration container.
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
    /// let value = sw.timestamp_type()?;
    /// dbg!(value);
    /// # Ok(()) }
    /// ```
    pub fn timestamp_type(&self) -> Result<TimestampType> {
        unsafe {
            let mut timestamp_type = mem::MaybeUninit::uninit();
            alsa::snd_pcm_sw_params_get_tstamp_type(
                self.handle.as_ptr(),
                timestamp_type.as_mut_ptr(),
            );
            let timestamp_type = timestamp_type.assume_init();
            let timestamp_type = TimestampType::from_value(timestamp_type)
                .ok_or_else(|| Error::BadTimestampType(timestamp_type))?;
            Ok(timestamp_type)
        }
    }

    /// Get avail min from a software configuration container.
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
    /// let available_min = sw.available_min()?;
    /// dbg!(available_min);
    /// # Ok(()) }
    /// ```
    pub fn available_min(&self) -> Result<c::c_ulong> {
        unsafe {
            let mut available_min = mem::MaybeUninit::uninit();
            alsa::snd_pcm_sw_params_get_avail_min(self.handle.as_ptr(), available_min.as_mut_ptr());
            Ok(available_min.assume_init())
        }
    }

    /// Get period event from a software configuration container.
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
    /// let value = sw.period_event()?;
    /// # Ok(()) }
    /// ```
    pub fn period_event(&self) -> Result<c::c_int> {
        unsafe {
            let mut value = mem::MaybeUninit::uninit();
            alsa::snd_pcm_sw_params_get_period_event(self.handle.as_ptr(), value.as_mut_ptr());
            Ok(value.assume_init())
        }
    }

    /// Get start threshold from a software configuration container.
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
    /// let value = sw.start_threshold()?;
    /// # Ok(()) }
    /// ```
    pub fn start_threshold(&self) -> Result<c::c_ulong> {
        unsafe {
            let mut value = mem::MaybeUninit::uninit();
            alsa::snd_pcm_sw_params_get_start_threshold(self.handle.as_ptr(), value.as_mut_ptr());
            Ok(value.assume_init())
        }
    }

    /// Get stop threshold from a software configuration container.
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
    /// let value = sw.stop_threshold()?;
    /// # Ok(()) }
    /// ```
    pub fn stop_threshold(&self) -> Result<c::c_ulong> {
        unsafe {
            let mut value = mem::MaybeUninit::uninit();
            alsa::snd_pcm_sw_params_get_stop_threshold(self.handle.as_ptr(), value.as_mut_ptr());
            Ok(value.assume_init())
        }
    }

    /// Get silence threshold from a software configuration container.
    ///
    /// A portion of playback buffer is overwritten with silence (see
    /// [set_silence_size][SoftwareParametersMut::set_silence_size]) when
    /// playback underrun is nearer than silence threshold.
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
    /// let value = sw.silence_threshold()?;
    /// # Ok(()) }
    /// ```
    pub fn silence_threshold(&self) -> Result<c::c_ulong> {
        unsafe {
            let mut value = mem::MaybeUninit::uninit();
            alsa::snd_pcm_sw_params_get_silence_threshold(self.handle.as_ptr(), value.as_mut_ptr());
            Ok(value.assume_init())
        }
    }

    /// Get silence size from a software configuration container.
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
    /// let value = sw.silence_size()?;
    /// # Ok(()) }
    /// ```
    pub fn silence_size(&self) -> Result<c::c_ulong> {
        unsafe {
            let mut value = mem::MaybeUninit::uninit();
            alsa::snd_pcm_sw_params_get_silence_size(self.handle.as_ptr(), value.as_mut_ptr());
            Ok(value.assume_init())
        }
    }
}

impl Drop for SoftwareParameters {
    fn drop(&mut self) {
        unsafe {
            alsa::snd_pcm_sw_params_free(self.handle.as_mut());
        }
    }
}

/// Collection of mutable software parameters being configured for a
/// [Pcm][super::Pcm] handle.
///
/// See
/// [Pcm::software_parameters_mut][super::Pcm::software_parameters_mut].
pub struct SoftwareParametersMut<'a> {
    pcm: &'a mut ptr::NonNull<alsa::snd_pcm_t>,
    base: SoftwareParameters,
}

impl<'a> SoftwareParametersMut<'a> {
    /// Open current software parameters for the current device for writing.
    pub(super) unsafe fn new(pcm: &'a mut ptr::NonNull<alsa::snd_pcm_t>) -> Result<Self> {
        let base = SoftwareParameters::new(pcm)?;

        Ok(Self { pcm, base })
    }

    /// Install PCM software configuration defined by params.
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
    pub fn install(mut self) -> Result<()> {
        unsafe {
            errno!(alsa::snd_pcm_sw_params(
                self.pcm.as_mut(),
                self.base.handle.as_mut()
            ))?;
            Ok(())
        }
    }

    /// Set timestamp mode inside a software configuration container.
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
    /// # Ok(()) }
    /// ```
    pub fn set_timestamp_mode(&mut self, timestamp_mode: Timestamp) -> Result<()> {
        unsafe {
            errno!(alsa::snd_pcm_sw_params_set_tstamp_mode(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                timestamp_mode as c::c_uint,
            ))?;
            Ok(())
        }
    }

    /// Set timestamp type inside a software configuration container.
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
    /// sw.set_timestamp_type(alsa::TimestampType::Monotonic)?;
    /// # Ok(()) }
    /// ```
    pub fn set_timestamp_type(&mut self, timestamp_type: TimestampType) -> Result<()> {
        unsafe {
            errno!(alsa::snd_pcm_sw_params_set_tstamp_type(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                timestamp_type as c::c_uint,
            ))?;
            Ok(())
        }
    }

    /// Set avail min inside a software configuration container.
    ///
    /// This is similar to setting an OSS wakeup point. The valid values for
    /// 'val' are determined by the specific hardware. Most PC sound cards can
    /// only accept power of 2 frame counts (i.e. 512, 1024, 2048). You cannot
    /// use this as a high resolution timer - it is limited to how often the
    /// sound card hardware raises an interrupt.
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
    /// sw.set_available_min(1000)?;
    /// # Ok(()) }
    /// ```
    pub fn set_available_min(&mut self, available_min: c::c_ulong) -> Result<()> {
        unsafe {
            errno!(alsa::snd_pcm_sw_params_set_avail_min(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                available_min
            ))?;
            Ok(())
        }
    }

    /// Set period event inside a software configuration container.
    ///
    /// An poll (select) wakeup event is raised if enabled.
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
    /// sw.set_period_event(0)?;
    /// # Ok(()) }
    /// ```
    pub fn set_period_event(&mut self, period_event: c::c_int) -> Result<()> {
        unsafe {
            errno!(alsa::snd_pcm_sw_params_set_period_event(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                period_event
            ))?;
            Ok(())
        }
    }

    /// Set start threshold inside a software configuration container.
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
    /// sw.set_start_threshold(0)?;
    /// # Ok(()) }
    /// ```
    pub fn set_start_threshold(&mut self, start_threshold: c::c_ulong) -> Result<()> {
        unsafe {
            errno!(alsa::snd_pcm_sw_params_set_start_threshold(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                start_threshold
            ))?;
            Ok(())
        }
    }

    /// Set stop threshold inside a software configuration container.
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
    /// sw.set_stop_threshold(0)?;
    /// # Ok(()) }
    /// ```
    pub fn set_stop_threshold(&mut self, stop_threshold: c::c_ulong) -> Result<()> {
        unsafe {
            errno!(alsa::snd_pcm_sw_params_set_stop_threshold(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                stop_threshold
            ))?;
            Ok(())
        }
    }

    /// Set silence threshold inside a software configuration container.
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
    /// sw.set_silence_threshold(0)?;
    /// # Ok(()) }
    /// ```
    pub fn set_silence_threshold(&mut self, silence_threshold: c::c_ulong) -> Result<()> {
        unsafe {
            errno!(alsa::snd_pcm_sw_params_set_silence_threshold(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                silence_threshold
            ))?;
            Ok(())
        }
    }

    /// Set silence size inside a software configuration container.
    ///
    /// A portion of playback buffer is overwritten with silence when playback
    /// underrun is nearer than silence threshold (see
    /// snd_pcm_sw_params_set_silence_threshold)
    ///
    /// The special case is when silence size value is equal or greater than
    /// boundary. The unused portion of the ring buffer (initial written samples
    /// are untouched) is filled with silence at start. Later, only just
    /// processed sample area is filled with silence. Note: silence_threshold
    /// must be set to zero.
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
    /// sw.set_silence_size(0)?;
    /// # Ok(()) }
    /// ```
    pub fn set_silence_size(&mut self, silence_size: c::c_ulong) -> Result<()> {
        unsafe {
            errno!(alsa::snd_pcm_sw_params_set_silence_size(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                silence_size
            ))?;
            Ok(())
        }
    }
}

impl ops::Deref for SoftwareParametersMut<'_> {
    type Target = SoftwareParameters;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
