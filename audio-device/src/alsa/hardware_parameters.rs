use crate::alsa::{Access, AccessMask, Error, Format, FormatMask, Result};
use crate::libc as c;
use alsa_sys as alsa;
use std::mem;
use std::ops;
use std::ptr;

/// The direction in which updated hardware parameters is restricted unless the
/// exact value is available.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
pub enum Direction {
    Smaller = -1,
    Nearest = 0,
    Greater = 1,
}

impl Direction {
    fn from_value(value: i32) -> Self {
        match value {
            -1 => Self::Smaller,
            0 => Self::Nearest,
            _ => Self::Greater,
        }
    }
}

/// Collection of current hardware parameters being configured for a
/// [Pcm][super::Pcm] handle.
///
/// See [Pcm::hardware_parameters][super::Pcm::hardware_parameters].
pub struct HardwareParameters {
    handle: ptr::NonNull<alsa::snd_pcm_hw_params_t>,
}

impl HardwareParameters {
    /// Open current hardware parameters for the current device for writing.
    pub(super) unsafe fn current(pcm: &mut ptr::NonNull<alsa::snd_pcm_t>) -> Result<Self> {
        let mut handle = mem::MaybeUninit::uninit();

        errno!(alsa::snd_pcm_hw_params_malloc(handle.as_mut_ptr()))?;

        let mut handle = ptr::NonNull::new_unchecked(handle.assume_init());

        if let Err(e) = errno!(alsa::snd_pcm_hw_params_current(
            pcm.as_ptr(),
            handle.as_mut()
        )) {
            alsa::snd_pcm_hw_params_free(handle.as_mut());
            return Err(e);
        }

        Ok(HardwareParameters { handle })
    }

    /// Open all available hardware parameters for the current device.
    pub(super) unsafe fn any(pcm: &mut ptr::NonNull<alsa::snd_pcm_t>) -> Result<Self> {
        let mut handle = mem::MaybeUninit::uninit();

        errno!(alsa::snd_pcm_hw_params_malloc(handle.as_mut_ptr()))?;

        let mut handle = ptr::NonNull::new_unchecked(handle.assume_init());

        if let Err(e) = errno!(alsa::snd_pcm_hw_params_any(pcm.as_ptr(), handle.as_mut())) {
            alsa::snd_pcm_hw_params_free(handle.as_mut());
            return Err(e);
        }

        Ok(HardwareParameters { handle })
    }

    /// Restrict a configuration space to contain only one channels count.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let hw = pcm.hardware_parameters()?;
    ///
    /// let result = hw.channels()?;
    /// dbg!(result);
    /// # Ok(()) }
    /// ```
    pub fn channels(&self) -> Result<c::c_uint> {
        unsafe {
            let mut channels = mem::MaybeUninit::uninit();

            errno!(alsa::snd_pcm_hw_params_get_channels(
                self.handle.as_ptr(),
                channels.as_mut_ptr()
            ))?;

            Ok(channels.assume_init())
        }
    }

    /// Extract maximum channels count from a configuration space.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let hw = pcm.hardware_parameters()?;
    ///
    /// println!("{}", hw.channels_max()?);
    /// # Ok(()) }
    /// ```
    pub fn channels_max(&self) -> Result<c::c_uint> {
        unsafe {
            let mut channels = mem::MaybeUninit::uninit();

            errno!(alsa::snd_pcm_hw_params_get_channels_max(
                self.handle.as_ptr(),
                channels.as_mut_ptr()
            ))?;

            Ok(channels.assume_init())
        }
    }

    /// Extract minimum channels count from a configuration space.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let hw = pcm.hardware_parameters()?;
    ///
    /// println!("{}", hw.channels_min()?);
    /// # Ok(()) }
    /// ```
    pub fn channels_min(&self) -> Result<c::c_uint> {
        unsafe {
            let mut channels = mem::MaybeUninit::uninit();

            errno!(alsa::snd_pcm_hw_params_get_channels_min(
                self.handle.as_ptr(),
                channels.as_mut_ptr()
            ))?;

            Ok(channels.assume_init())
        }
    }

    /// Extract rate from a configuration space.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// println!("{}", hw.rate()?);
    /// # Ok(()) }
    /// ```
    pub fn rate(&self) -> Result<c::c_uint> {
        unsafe {
            let mut rate = 0;
            let mut dir = 0;

            errno!(alsa::snd_pcm_hw_params_get_rate(
                self.handle.as_ptr(),
                &mut rate,
                &mut dir,
            ))?;

            Ok(rate)
        }
    }

    /// Get rate exact info from a configuration space.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// dbg!(hw.rate_numden()?);
    /// # Ok(()) }
    /// ```
    pub fn rate_numden(&self) -> Result<(c::c_uint, c::c_uint)> {
        unsafe {
            let mut num = mem::MaybeUninit::uninit();
            let mut den = mem::MaybeUninit::uninit();
            errno!(alsa::snd_pcm_hw_params_get_rate_numden(
                self.handle.as_ptr(),
                num.as_mut_ptr(),
                den.as_mut_ptr(),
            ))?;
            let num = num.assume_init();
            let den = den.assume_init();
            Ok((num, den))
        }
    }

    /// Extract max rate from a configuration space.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// println!("{}", hw.rate_max()?);
    /// # Ok(()) }
    /// ```
    pub fn rate_max(&self) -> Result<c::c_uint> {
        unsafe {
            let mut rate = 0;
            let mut dir = 0;

            errno!(alsa::snd_pcm_hw_params_get_rate_max(
                self.handle.as_ptr(),
                &mut rate,
                &mut dir,
            ))?;

            Ok(rate)
        }
    }

    /// Extract min rate from a configuration space.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// println!("{}", hw.rate_min()?);
    /// # Ok(()) }
    /// ```
    pub fn rate_min(&self) -> Result<c::c_uint> {
        unsafe {
            let mut rate = 0;
            let mut dir = 0;

            errno!(alsa::snd_pcm_hw_params_get_rate_min(
                self.handle.as_ptr(),
                &mut rate,
                &mut dir,
            ))?;

            Ok(rate)
        }
    }

    /// Extract format from a configuration space.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// println!("{}", hw.format()?);
    /// # Ok(()) }
    /// ```
    pub fn format(&self) -> Result<Format> {
        unsafe {
            let mut format = mem::MaybeUninit::uninit();

            errno!(alsa::snd_pcm_hw_params_get_format(
                self.handle.as_ptr(),
                format.as_mut_ptr(),
            ))?;

            let format = format.assume_init();
            let format = Format::from_value(format).ok_or_else(|| Error::BadFormat(format))?;
            Ok(format)
        }
    }

    /// Get format mask from a configuration space.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let hw = pcm.hardware_parameters()?;
    ///
    /// let _mask = hw.format_mask()?;
    /// # Ok(()) }
    /// ```
    pub fn format_mask(&self) -> Result<FormatMask> {
        unsafe {
            let mut mask = FormatMask::allocate()?;
            alsa::snd_pcm_hw_params_get_format_mask(self.handle.as_ptr(), mask.handle.as_mut());
            Ok(mask)
        }
    }

    /// Extract access type from a configuration space.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// println!("{}", hw.access()?);
    /// # Ok(()) }
    /// ```
    pub fn access(&self) -> Result<Access> {
        unsafe {
            let mut access = mem::MaybeUninit::uninit();

            errno!(alsa::snd_pcm_hw_params_get_access(
                self.handle.as_ptr(),
                access.as_mut_ptr(),
            ))?;

            let access = access.assume_init();
            let access = Access::from_value(access).ok_or_else(|| Error::BadAccess(access))?;
            Ok(access)
        }
    }

    /// Get access mask from a configuration space.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let hw = pcm.hardware_parameters()?;
    ///
    /// let _mask = hw.get_access_mask()?;
    /// # Ok(()) }
    /// ```
    pub fn get_access_mask(&self) -> Result<AccessMask> {
        unsafe {
            let mut mask = AccessMask::allocate()?;

            errno!(alsa::snd_pcm_hw_params_get_access_mask(
                self.handle.as_ptr(),
                mask.handle.as_mut(),
            ))?;

            Ok(mask)
        }
    }

    /// Check if hardware supports pause.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// println!("{}", hw.can_pause());
    /// # Ok(()) }
    /// ```
    pub fn can_pause(&self) -> bool {
        unsafe { alsa::snd_pcm_hw_params_can_pause(self.handle.as_ptr()) != 0 }
    }

    /// Check if hardware supports resume.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// println!("{}", hw.can_resume());
    /// # Ok(()) }
    /// ```
    pub fn can_resume(&self) -> bool {
        unsafe { alsa::snd_pcm_hw_params_can_resume(self.handle.as_ptr()) != 0 }
    }

    /// Copy one hardware parameters to another.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let a = pcm.hardware_parameters()?;
    /// let mut hw = pcm.hardware_parameters()?;
    ///
    /// hw.copy(&a);
    /// # Ok(()) }
    /// ```
    pub fn copy(&mut self, other: &HardwareParameters) {
        unsafe { alsa::snd_pcm_hw_params_copy(self.handle.as_mut(), other.handle.as_ptr()) };
    }

    /// Check if hardware supports sample-resolution mmap for given configuration.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let hw = pcm.hardware_parameters()?;
    ///
    /// println!("{}", hw.can_mmap_sample_resolution());
    /// # Ok(()) }
    /// ```
    pub fn can_mmap_sample_resolution(&self) -> bool {
        unsafe { alsa::snd_pcm_hw_params_can_mmap_sample_resolution(self.handle.as_ptr()) == 1 }
    }

    /// Check if hardware does double buffering for start/stop for given configuration.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let hw = pcm.hardware_parameters()?;
    ///
    /// println!("{}", hw.is_double());
    /// # Ok(()) }
    /// ```
    pub fn is_double(&self) -> bool {
        unsafe { alsa::snd_pcm_hw_params_is_double(self.handle.as_ptr()) == 1 }
    }

    /// Check if hardware does double buffering for data transfers for given configuration.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let hw = pcm.hardware_parameters()?;
    ///
    /// println!("{}", hw.is_batch());
    /// # Ok(()) }
    /// ```
    pub fn is_batch(&self) -> bool {
        unsafe { alsa::snd_pcm_hw_params_is_batch(self.handle.as_ptr()) == 1 }
    }

    /// Check if hardware does block transfers for samples for given configuration.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let hw = pcm.hardware_parameters()?;
    ///
    /// println!("{}", hw.is_block_transfer());
    /// # Ok(()) }
    /// ```
    pub fn is_block_transfer(&self) -> bool {
        unsafe { alsa::snd_pcm_hw_params_is_block_transfer(self.handle.as_ptr()) == 1 }
    }

    /// Check if timestamps are monotonic for given configuration.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let hw = pcm.hardware_parameters()?;
    ///
    /// println!("{}", hw.is_monotonic());
    /// # Ok(()) }
    /// ```
    pub fn is_monotonic(&self) -> bool {
        unsafe { alsa::snd_pcm_hw_params_is_monotonic(self.handle.as_ptr()) == 1 }
    }

    /// Check if hardware supports overrange detection.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let hw = pcm.hardware_parameters()?;
    ///
    /// println!("{}", hw.can_overrange());
    /// # Ok(()) }
    /// ```
    pub fn can_overrange(&self) -> bool {
        unsafe { alsa::snd_pcm_hw_params_can_overrange(self.handle.as_ptr()) == 1 }
    }

    /// Check if hardware does half-duplex only.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let hw = pcm.hardware_parameters()?;
    ///
    /// println!("{}", hw.is_half_duplex());
    /// # Ok(()) }
    /// ```
    pub fn is_half_duplex(&self) -> bool {
        unsafe { alsa::snd_pcm_hw_params_is_half_duplex(self.handle.as_ptr()) == 1 }
    }

    /// Check if hardware does joint-duplex (playback and capture are somewhat correlated)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let hw = pcm.hardware_parameters()?;
    ///
    /// println!("{}", hw.is_joint_duplex());
    /// # Ok(()) }
    /// ```
    pub fn is_joint_duplex(&self) -> bool {
        unsafe { alsa::snd_pcm_hw_params_is_joint_duplex(self.handle.as_ptr()) == 1 }
    }

    /// Check if hardware supports synchronized start with sample resolution.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let hw = pcm.hardware_parameters()?;
    ///
    /// println!("{}", hw.can_sync_start());
    /// # Ok(()) }
    /// ```
    pub fn can_sync_start(&self) -> bool {
        unsafe { alsa::snd_pcm_hw_params_can_sync_start(self.handle.as_ptr()) == 1 }
    }

    /// Check if hardware can disable period wakeups.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let hw = pcm.hardware_parameters()?;
    ///
    /// println!("{}", hw.can_disable_period_wakeup());
    /// # Ok(()) }
    /// ```
    pub fn can_disable_period_wakeup(&self) -> bool {
        unsafe { alsa::snd_pcm_hw_params_can_disable_period_wakeup(self.handle.as_ptr()) == 1 }
    }

    /// Check if hardware supports audio wallclock timestamps.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let hw = pcm.hardware_parameters()?;
    ///
    /// println!("{}", hw.supports_audio_wallclock_ts());
    /// # Ok(()) }
    /// ```
    pub fn supports_audio_wallclock_ts(&self) -> bool {
        unsafe { alsa::snd_pcm_hw_params_supports_audio_wallclock_ts(self.handle.as_ptr()) == 1 }
    }

    /// Check if hardware supports type of audio timestamps.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let hw = pcm.hardware_parameters()?;
    ///
    /// println!("{}", hw.supports_audio_ts_type(2));
    /// # Ok(()) }
    /// ```
    pub fn supports_audio_ts_type(&self, ty: c::c_int) -> bool {
        unsafe { alsa::snd_pcm_hw_params_supports_audio_ts_type(self.handle.as_ptr(), ty) == 1 }
    }

    /// Get sample resolution info from a configuration space.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let hw = pcm.hardware_parameters()?;
    ///
    /// println!("{}", hw.sbits()?);
    /// # Ok(()) }
    /// ```
    pub fn sbits(&self) -> Result<c::c_int> {
        unsafe { errno!(alsa::snd_pcm_hw_params_get_sbits(self.handle.as_ptr())) }
    }

    /// Get hardware FIFO size info from a configuration space.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let hw = pcm.hardware_parameters()?;
    ///
    /// println!("{}", hw.fifo_size()?);
    /// # Ok(()) }
    /// ```
    pub fn fifo_size(&self) -> Result<c::c_int> {
        unsafe { errno!(alsa::snd_pcm_hw_params_get_fifo_size(self.handle.as_ptr())) }
    }

    /// Extract period time from a configuration space.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let hw = pcm.hardware_parameters()?;
    ///
    /// dbg!(hw.period_time());
    /// # Ok(()) }
    /// ```
    pub fn period_time(&self) -> Result<(c::c_uint, Direction)> {
        unsafe {
            let mut period_time = mem::MaybeUninit::uninit();
            let mut dir = mem::MaybeUninit::uninit();
            alsa::snd_pcm_hw_params_get_period_time(
                self.handle.as_ptr(),
                period_time.as_mut_ptr(),
                dir.as_mut_ptr(),
            );
            let period_time = period_time.assume_init();
            let dir = Direction::from_value(dir.assume_init());
            Ok((period_time, dir))
        }
    }

    /// Extract minimum period time from a configuration space.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let hw = pcm.hardware_parameters()?;
    ///
    /// dbg!(hw.period_time_min());
    /// # Ok(()) }
    /// ```
    pub fn period_time_min(&self) -> Result<(c::c_uint, Direction)> {
        unsafe {
            let mut period_time = mem::MaybeUninit::uninit();
            let mut dir = mem::MaybeUninit::uninit();
            alsa::snd_pcm_hw_params_get_period_time_min(
                self.handle.as_ptr(),
                period_time.as_mut_ptr(),
                dir.as_mut_ptr(),
            );
            let period_time = period_time.assume_init();
            let dir = Direction::from_value(dir.assume_init());
            Ok((period_time, dir))
        }
    }

    /// Extract maximum period time from a configuration space.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let hw = pcm.hardware_parameters()?;
    ///
    /// dbg!(hw.period_time_max());
    /// # Ok(()) }
    /// ```
    pub fn period_time_max(&self) -> Result<(c::c_uint, Direction)> {
        unsafe {
            let mut period_time = mem::MaybeUninit::uninit();
            let mut dir = mem::MaybeUninit::uninit();
            alsa::snd_pcm_hw_params_get_period_time_max(
                self.handle.as_ptr(),
                period_time.as_mut_ptr(),
                dir.as_mut_ptr(),
            );
            let period_time = period_time.assume_init();
            let dir = Direction::from_value(dir.assume_init());
            Ok((period_time, dir))
        }
    }

    /// Extract period size from a configuration space.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let hw = pcm.hardware_parameters()?;
    ///
    /// dbg!(hw.period_size());
    /// # Ok(()) }
    /// ```
    pub fn period_size(&self) -> Result<(c::c_ulong, Direction)> {
        unsafe {
            let mut frames = mem::MaybeUninit::uninit();
            let mut dir = mem::MaybeUninit::uninit();
            errno!(alsa::snd_pcm_hw_params_get_period_size(
                self.handle.as_ptr(),
                frames.as_mut_ptr(),
                dir.as_mut_ptr()
            ))?;
            let frames = frames.assume_init();
            let dir = dir.assume_init();
            let dir = Direction::from_value(dir);
            Ok((frames, dir))
        }
    }

    /// Extract minimum period size from a configuration space.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let hw = pcm.hardware_parameters()?;
    ///
    /// dbg!(hw.period_size_min());
    /// # Ok(()) }
    /// ```
    pub fn period_size_min(&self) -> Result<(c::c_ulong, Direction)> {
        unsafe {
            let mut frames = mem::MaybeUninit::uninit();
            let mut dir = mem::MaybeUninit::uninit();
            errno!(alsa::snd_pcm_hw_params_get_period_size_min(
                self.handle.as_ptr(),
                frames.as_mut_ptr(),
                dir.as_mut_ptr()
            ))?;
            let frames = frames.assume_init();
            let dir = dir.assume_init();
            let dir = Direction::from_value(dir);
            Ok((frames, dir))
        }
    }

    /// Extract maximum period size from a configuration space.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let hw = pcm.hardware_parameters()?;
    ///
    /// dbg!(hw.period_size_max()?);
    /// # Ok(()) }
    /// ```
    pub fn period_size_max(&self) -> Result<(c::c_ulong, Direction)> {
        unsafe {
            let mut frames = mem::MaybeUninit::uninit();
            let mut dir = mem::MaybeUninit::uninit();
            errno!(alsa::snd_pcm_hw_params_get_period_size_max(
                self.handle.as_ptr(),
                frames.as_mut_ptr(),
                dir.as_mut_ptr()
            ))?;
            let frames = frames.assume_init();
            let dir = dir.assume_init();
            let dir = Direction::from_value(dir);
            Ok((frames, dir))
        }
    }

    /// Extract periods from a configuration space.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let hw = pcm.hardware_parameters()?;
    ///
    /// dbg!(hw.periods()?);
    /// # Ok(()) }
    /// ```
    pub fn periods(&self) -> Result<(c::c_uint, Direction)> {
        unsafe {
            let mut periods = mem::MaybeUninit::uninit();
            let mut dir = mem::MaybeUninit::uninit();
            errno!(alsa::snd_pcm_hw_params_get_periods(
                self.handle.as_ptr(),
                periods.as_mut_ptr(),
                dir.as_mut_ptr(),
            ))?;
            let periods = periods.assume_init();
            let dir = Direction::from_value(dir.assume_init());
            Ok((periods, dir))
        }
    }

    /// Extract minimum periods count from a configuration space.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let hw = pcm.hardware_parameters()?;
    ///
    /// dbg!(hw.periods_min()?);
    /// # Ok(()) }
    /// ```
    pub fn periods_min(&self) -> Result<(c::c_uint, Direction)> {
        unsafe {
            let mut periods = mem::MaybeUninit::uninit();
            let mut dir = mem::MaybeUninit::uninit();
            errno!(alsa::snd_pcm_hw_params_get_periods_min(
                self.handle.as_ptr(),
                periods.as_mut_ptr(),
                dir.as_mut_ptr(),
            ))?;
            let periods = periods.assume_init();
            let dir = Direction::from_value(dir.assume_init());
            Ok((periods, dir))
        }
    }

    /// Extract maximum periods count from a configuration space.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let hw = pcm.hardware_parameters()?;
    ///
    /// dbg!(hw.periods_max()?);
    /// # Ok(()) }
    /// ```
    pub fn periods_max(&self) -> Result<(c::c_uint, Direction)> {
        unsafe {
            let mut periods = mem::MaybeUninit::uninit();
            let mut dir = mem::MaybeUninit::uninit();
            errno!(alsa::snd_pcm_hw_params_get_periods_max(
                self.handle.as_ptr(),
                periods.as_mut_ptr(),
                dir.as_mut_ptr(),
            ))?;
            let periods = periods.assume_init();
            let dir = Direction::from_value(dir.assume_init());
            Ok((periods, dir))
        }
    }

    /// Extract buffer time from a configuration space.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let hw = pcm.hardware_parameters()?;
    ///
    /// dbg!(hw.buffer_time()?);
    /// # Ok(()) }
    /// ```
    pub fn buffer_time(&self) -> Result<(c::c_uint, Direction)> {
        unsafe {
            let mut periods = mem::MaybeUninit::uninit();
            let mut dir = mem::MaybeUninit::uninit();
            errno!(alsa::snd_pcm_hw_params_get_buffer_time(
                self.handle.as_ptr(),
                periods.as_mut_ptr(),
                dir.as_mut_ptr(),
            ))?;
            let periods = periods.assume_init();
            let dir = Direction::from_value(dir.assume_init());
            Ok((periods, dir))
        }
    }

    /// Extract minimum buffer time from a configuration space.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let hw = pcm.hardware_parameters()?;
    ///
    /// dbg!(hw.buffer_time_min()?);
    /// # Ok(()) }
    /// ```
    pub fn buffer_time_min(&self) -> Result<(c::c_uint, Direction)> {
        unsafe {
            let mut periods = mem::MaybeUninit::uninit();
            let mut dir = mem::MaybeUninit::uninit();
            errno!(alsa::snd_pcm_hw_params_get_buffer_time_min(
                self.handle.as_ptr(),
                periods.as_mut_ptr(),
                dir.as_mut_ptr(),
            ))?;
            let periods = periods.assume_init();
            let dir = Direction::from_value(dir.assume_init());
            Ok((periods, dir))
        }
    }

    /// Extract maximum buffer time from a configuration space.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let hw = pcm.hardware_parameters()?;
    ///
    /// dbg!(hw.buffer_time_max()?);
    /// # Ok(()) }
    /// ```
    pub fn buffer_time_max(&self) -> Result<(c::c_uint, Direction)> {
        unsafe {
            let mut periods = mem::MaybeUninit::uninit();
            let mut dir = mem::MaybeUninit::uninit();
            errno!(alsa::snd_pcm_hw_params_get_buffer_time_max(
                self.handle.as_ptr(),
                periods.as_mut_ptr(),
                dir.as_mut_ptr(),
            ))?;
            let periods = periods.assume_init();
            let dir = Direction::from_value(dir.assume_init());
            Ok((periods, dir))
        }
    }

    /// Extract buffer size from a configuration space.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters()?;
    ///
    /// dbg!(hw.buffer_size()?);
    /// # Ok(()) }
    /// ```
    pub fn buffer_size(&self) -> Result<c::c_ulong> {
        unsafe {
            let mut buffer_size = mem::MaybeUninit::uninit();
            errno!(alsa::snd_pcm_hw_params_get_buffer_size(
                self.handle.as_ptr(),
                buffer_size.as_mut_ptr()
            ))?;
            Ok(buffer_size.assume_init())
        }
    }

    /// Extract minimum buffer size from a configuration space.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters()?;
    ///
    /// dbg!(hw.buffer_size_min()?);
    /// # Ok(()) }
    /// ```
    pub fn buffer_size_min(&self) -> Result<c::c_ulong> {
        unsafe {
            let mut buffer_size = mem::MaybeUninit::uninit();
            errno!(alsa::snd_pcm_hw_params_get_buffer_size_min(
                self.handle.as_ptr(),
                buffer_size.as_mut_ptr()
            ))?;
            Ok(buffer_size.assume_init())
        }
    }

    /// Extract maximum buffer size from a configuration space.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters()?;
    ///
    /// dbg!(hw.buffer_size_max()?);
    /// # Ok(()) }
    /// ```
    pub fn buffer_size_max(&self) -> Result<c::c_ulong> {
        unsafe {
            let mut buffer_size = mem::MaybeUninit::uninit();
            errno!(alsa::snd_pcm_hw_params_get_buffer_size_max(
                self.handle.as_ptr(),
                buffer_size.as_mut_ptr()
            ))?;
            Ok(buffer_size.assume_init())
        }
    }

    /// Get the minimum transfer align value in samples.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters()?;
    ///
    /// dbg!(hw.min_align()?);
    /// # Ok(()) }
    /// ```
    pub fn min_align(&self) -> Result<c::c_ulong> {
        unsafe {
            let mut min_align = mem::MaybeUninit::uninit();
            errno!(alsa::snd_pcm_hw_params_get_min_align(
                self.handle.as_ptr(),
                min_align.as_mut_ptr()
            ))?;
            Ok(min_align.assume_init())
        }
    }
}

impl Drop for HardwareParameters {
    fn drop(&mut self) {
        unsafe {
            let _ = alsa::snd_pcm_hw_params_free(self.handle.as_mut());
        }
    }
}

/// Collection of harward parameters being configured for a [Pcm][super::Pcm]
/// handle.
///
/// Must be refined before they are applied to a [Pcm][super::Pcm] device
/// through [HardwareParametersMut::install].
///
/// See [Pcm::hardware_parameters_any][super::Pcm::hardware_parameters_any].
pub struct HardwareParametersMut<'a> {
    pcm: &'a mut ptr::NonNull<alsa::snd_pcm_t>,
    base: HardwareParameters,
}

impl<'a> HardwareParametersMut<'a> {
    /// Open current hardware parameters for the current device for writing.
    pub(super) unsafe fn current(pcm: &'a mut ptr::NonNull<alsa::snd_pcm_t>) -> Result<Self> {
        let base = HardwareParameters::current(pcm)?;
        Ok(HardwareParametersMut { pcm, base })
    }

    /// Open all available hardware parameters for the current device.
    pub(super) unsafe fn any(pcm: &'a mut ptr::NonNull<alsa::snd_pcm_t>) -> Result<Self> {
        let base = HardwareParameters::any(pcm)?;
        Ok(HardwareParametersMut { pcm, base })
    }

    /// Install one PCM hardware configuration chosen from a configuration space
    /// and prepare it.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// hw.set_channels_near(2)?;
    /// hw.install()?;
    /// # Ok(()) }
    /// ```
    pub fn install(mut self) -> Result<()> {
        unsafe {
            errno!(alsa::snd_pcm_hw_params(
                self.pcm.as_mut(),
                self.base.handle.as_mut()
            ))?;
            Ok(())
        }
    }

    /// Extract resample state from a configuration space.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let result = hw.rate_resample()?;
    /// dbg!(result);
    /// # Ok(()) }
    /// ```
    pub fn rate_resample(&mut self) -> Result<bool> {
        unsafe {
            let mut v = mem::MaybeUninit::uninit();

            errno!(alsa::snd_pcm_hw_params_get_rate_resample(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                v.as_mut_ptr()
            ))?;

            Ok(v.assume_init() != 0)
        }
    }

    /// Restrict a configuration space to contain only real hardware rates.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_rate_resample(true)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_rate_resample(&mut self, resample: bool) -> Result<()> {
        unsafe {
            errno!(alsa::snd_pcm_hw_params_set_rate_resample(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                if resample { 1 } else { 0 }
            ))?;

            Ok(())
        }
    }

    /// Restrict a configuration space to have channels count nearest to a target.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_channels_near(2)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_channels_near(&mut self, mut channels: c::c_uint) -> Result<c::c_uint> {
        unsafe {
            errno!(alsa::snd_pcm_hw_params_set_channels_near(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                &mut channels
            ))?;

            Ok(channels)
        }
    }

    /// Restrict a configuration space to contain only one channels count.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_channels(2)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_channels(&mut self, channels: c::c_uint) -> Result<()> {
        unsafe {
            errno!(alsa::snd_pcm_hw_params_set_channels(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                channels
            ))?;

            Ok(())
        }
    }

    /// Extract minimum channels count from a configuration space.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let result = hw.test_channels(4)?;
    /// dbg!(result);
    /// # Ok(()) }
    /// ```
    pub fn test_channels(&mut self, channels: c::c_uint) -> Result<bool> {
        unsafe {
            let result = alsa::snd_pcm_hw_params_test_channels(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                channels,
            );

            Ok(result == 0)
        }
    }

    /// Restrict a configuration space with a minimum channels count.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_channels_min(2)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_channels_min(&mut self, mut channels: c::c_uint) -> Result<c::c_uint> {
        unsafe {
            errno!(alsa::snd_pcm_hw_params_set_channels_min(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                &mut channels
            ))?;
            Ok(channels)
        }
    }

    /// Restrict a configuration space with a maximum channels count.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_channels_max(2)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_channels_max(&mut self, mut channels: c::c_uint) -> Result<c::c_uint> {
        unsafe {
            errno!(alsa::snd_pcm_hw_params_set_channels_max(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                &mut channels
            ))?;
            Ok(channels)
        }
    }

    /// Restrict a configuration space to have channels counts in a given range.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_channels_minmax(2, 4)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_channels_minmax(
        &mut self,
        mut channels_min: c::c_uint,
        mut channels_max: c::c_uint,
    ) -> Result<(c::c_uint, c::c_uint)> {
        unsafe {
            errno!(alsa::snd_pcm_hw_params_set_channels_minmax(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                &mut channels_min,
                &mut channels_max
            ))?;
            Ok((channels_min, channels_max))
        }
    }

    /// Restrict a configuration space to contain only its minimum channels count.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_channels_first()?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_channels_first(&mut self) -> Result<c::c_uint> {
        unsafe {
            let mut channels = mem::MaybeUninit::uninit();
            errno!(alsa::snd_pcm_hw_params_set_channels_first(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                channels.as_mut_ptr()
            ))?;
            Ok(channels.assume_init())
        }
    }

    /// Restrict a configuration space to contain only its maximum channels count.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_channels_last()?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_channels_last(&mut self) -> Result<c::c_uint> {
        unsafe {
            let mut channels = mem::MaybeUninit::uninit();
            errno!(alsa::snd_pcm_hw_params_set_channels_last(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                channels.as_mut_ptr()
            ))?;
            Ok(channels.assume_init())
        }
    }

    /// Restrict a configuration space to have rate nearest to a target.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_rate_near(44100, alsa::Direction::Nearest)?;
    /// dbg!(actual);
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_rate_near(
        &mut self,
        mut rate: c::c_uint,
        dir: Direction,
    ) -> Result<(u32, Direction)> {
        unsafe {
            let mut dir = dir as c::c_int;

            errno!(alsa::snd_pcm_hw_params_set_rate_near(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                &mut rate,
                &mut dir,
            ))?;
            let dir = Direction::from_value(dir);
            Ok((rate, dir))
        }
    }

    /// Restrict a configuration space to have rate nearest to a target.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_rate(44100, alsa::Direction::Nearest)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_rate(&mut self, rate: c::c_uint, dir: Direction) -> Result<()> {
        unsafe {
            errno!(alsa::snd_pcm_hw_params_set_rate(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                rate,
                dir as c::c_int,
            ))?;

            Ok(())
        }
    }

    /// Restrict a configuration space with a minimum rate.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_rate_min(44100, alsa::Direction::Nearest)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_rate_min(
        &mut self,
        mut rate: c::c_uint,
        dir: Direction,
    ) -> Result<(c::c_uint, Direction)> {
        unsafe {
            let mut dir = dir as i32;
            errno!(alsa::snd_pcm_hw_params_set_rate_min(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                &mut rate,
                &mut dir,
            ))?;
            let dir = Direction::from_value(dir);
            Ok((rate, dir))
        }
    }

    /// Restrict a configuration space with a maximum rate.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_rate_max(44100, alsa::Direction::Nearest)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_rate_max(
        &mut self,
        mut rate: c::c_uint,
        dir: Direction,
    ) -> Result<(c::c_uint, Direction)> {
        unsafe {
            let mut dir = dir as i32;
            errno!(alsa::snd_pcm_hw_params_set_rate_max(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                &mut rate,
                &mut dir,
            ))?;
            let dir = Direction::from_value(dir);
            Ok((rate, dir))
        }
    }

    /// Restrict a configuration space to have rates in a given range.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_rate_minmax(128, alsa::Direction::Nearest, 256, alsa::Direction::Nearest)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_rate_minmax(
        &mut self,
        mut rate_min: c::c_uint,
        dir_min: Direction,
        mut rate_max: c::c_uint,
        dir_max: Direction,
    ) -> Result<(c::c_uint, Direction, c::c_uint, Direction)> {
        unsafe {
            let mut dir_min = dir_min as i32;
            let mut dir_max = dir_max as i32;
            errno!(alsa::snd_pcm_hw_params_set_rate_minmax(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                &mut rate_min,
                &mut dir_min,
                &mut rate_max,
                &mut dir_max,
            ))?;
            let dir_min = Direction::from_value(dir_min);
            let dir_max = Direction::from_value(dir_max);
            Ok((rate_min, dir_min, rate_max, dir_max))
        }
    }

    /// Restrict a configuration space to contain only its minimum rate.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_rate_first()?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_rate_first(&mut self) -> Result<(c::c_uint, Direction)> {
        unsafe {
            let mut rate = mem::MaybeUninit::uninit();
            let mut dir = mem::MaybeUninit::uninit();
            errno!(alsa::snd_pcm_hw_params_set_rate_first(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                rate.as_mut_ptr(),
                dir.as_mut_ptr(),
            ))?;
            let rate = rate.assume_init();
            let dir = Direction::from_value(dir.assume_init());
            Ok((rate, dir))
        }
    }

    /// Restrict a configuration space to contain only its maximum rate.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_rate_last()?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_rate_last(&mut self) -> Result<(c::c_uint, Direction)> {
        unsafe {
            let mut rate = mem::MaybeUninit::uninit();
            let mut dir = mem::MaybeUninit::uninit();
            errno!(alsa::snd_pcm_hw_params_set_rate_last(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                rate.as_mut_ptr(),
                dir.as_mut_ptr(),
            ))?;
            let rate = rate.assume_init();
            let dir = Direction::from_value(dir.assume_init());
            Ok((rate, dir))
        }
    }

    /// Extract min rate from a configuration space.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let result = hw.test_rate(44100)?;
    /// dbg!(result);
    /// # Ok(()) }
    /// ```
    pub fn test_rate(&mut self, rate: c::c_uint) -> Result<bool> {
        unsafe {
            let result = alsa::snd_pcm_hw_params_test_rate(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                rate,
                0,
            );

            Ok(result == 0)
        }
    }

    /// Restrict a configuration space to contain only one format.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_format(alsa::Format::S16LE)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_format(&mut self, format: Format) -> Result<()> {
        unsafe {
            errno!(alsa::snd_pcm_hw_params_set_format(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                format as c::c_int
            ))?;

            Ok(())
        }
    }

    /// Restrict a configuration space to contain only its first format.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_format_first()?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_format_first(&mut self) -> Result<Format> {
        unsafe {
            let mut format = mem::MaybeUninit::uninit();

            errno!(alsa::snd_pcm_hw_params_set_format_first(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                format.as_mut_ptr(),
            ))?;

            let format = format.assume_init();
            let format = Format::from_value(format).ok_or_else(|| Error::BadFormat(format))?;
            Ok(format)
        }
    }

    /// Restrict a configuration space to contain only its last format.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_format_last()?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_format_last(&mut self) -> Result<Format> {
        unsafe {
            let mut format = mem::MaybeUninit::uninit();

            errno!(alsa::snd_pcm_hw_params_set_format_last(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                format.as_mut_ptr(),
            ))?;

            let format = format.assume_init();
            let format = Format::from_value(format).ok_or_else(|| Error::BadFormat(format))?;
            Ok(format)
        }
    }

    /// Restrict a configuration space to contain only a set of formats.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let mut mask = alsa::FormatMask::new()?;
    /// mask.set(alsa::Format::S16LE);
    ///
    /// let actual = hw.set_format_mask(&mask)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_format_mask(&mut self, mask: &FormatMask) -> Result<()> {
        unsafe {
            errno!(alsa::snd_pcm_hw_params_set_format_mask(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                mask.handle.as_ptr(),
            ))?;

            Ok(())
        }
    }

    /// Verify if a format is available inside a configuration space for a PCM.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let result = hw.test_format(alsa::Format::S16LE)?;
    /// dbg!(result);
    /// # Ok(()) }
    /// ```
    pub fn test_format(&mut self, format: Format) -> Result<bool> {
        unsafe {
            let result = alsa::snd_pcm_hw_params_test_format(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                format as c::c_int,
            );

            Ok(result == 0)
        }
    }

    /// Restrict a configuration space to contain only one access type.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_access(alsa::Access::MmapInterleaved)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_access(&mut self, access: Access) -> Result<()> {
        unsafe {
            errno!(alsa::snd_pcm_hw_params_set_access(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                access as c::c_uint
            ))?;

            Ok(())
        }
    }

    /// Verify if an access type is available inside a configuration space for a PCM.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let result = hw.test_access(alsa::Access::MmapInterleaved)?;
    /// dbg!(result);
    /// # Ok(()) }
    /// ```
    pub fn test_access(&mut self, access: Access) -> Result<bool> {
        unsafe {
            let result = alsa::snd_pcm_hw_params_test_access(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                access as c::c_uint,
            );

            Ok(result == 0)
        }
    }

    /// Restrict a configuration space to contain only its first access type.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// println!("{}", hw.set_access_first()?);
    /// # Ok(()) }
    /// ```
    pub fn set_access_first(&mut self) -> Result<Access> {
        unsafe {
            let mut access = mem::MaybeUninit::uninit();

            errno!(alsa::snd_pcm_hw_params_set_access_first(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                access.as_mut_ptr(),
            ))?;

            let access = access.assume_init();
            let access = Access::from_value(access).ok_or_else(|| Error::BadAccess(access))?;
            Ok(access)
        }
    }

    /// Restrict a configuration space to contain only its last access type.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// println!("{}", hw.set_access_last()?);
    /// # Ok(()) }
    /// ```
    pub fn set_access_last(&mut self) -> Result<Access> {
        unsafe {
            let mut access = mem::MaybeUninit::uninit();

            errno!(alsa::snd_pcm_hw_params_set_access_last(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                access.as_mut_ptr(),
            ))?;

            let access = access.assume_init();
            let access = Access::from_value(access).ok_or_else(|| Error::BadAccess(access))?;
            Ok(access)
        }
    }

    /// Restrict a configuration space to contain only a set of access types.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let mut mask = alsa::AccessMask::new()?;
    /// mask.set(alsa::Access::MmapInterleaved);
    ///
    /// let actual = hw.set_access_mask(&mask)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_access_mask(&mut self, mask: &AccessMask) -> Result<()> {
        unsafe {
            errno!(alsa::snd_pcm_hw_params_set_access_mask(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                mask.handle.as_ptr(),
            ))?;

            Ok(())
        }
    }

    /// Restrict a configuration space to allow the buffer to be accessible from outside.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_export_buffer(1024)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_export_buffer(&mut self, export_buffer: c::c_uint) -> Result<()> {
        unsafe {
            errno!(alsa::snd_pcm_hw_params_set_export_buffer(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                export_buffer
            ))?;
            Ok(())
        }
    }

    /// Extract buffer accessibility from a configuration space.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let result = hw.export_buffer()?;
    /// dbg!(result);
    /// # Ok(()) }
    /// ```
    pub fn export_buffer(&mut self) -> Result<c::c_uint> {
        unsafe {
            let mut export_buffer = mem::MaybeUninit::uninit();
            errno!(alsa::snd_pcm_hw_params_get_export_buffer(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                export_buffer.as_mut_ptr()
            ))?;
            Ok(export_buffer.assume_init())
        }
    }

    /// Restrict a configuration space to settings without period wakeups.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_period_wakeup(10000)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_period_wakeup(&mut self, period_wakeup: c::c_uint) -> Result<()> {
        unsafe {
            errno!(alsa::snd_pcm_hw_params_set_period_wakeup(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                period_wakeup
            ))?;
            Ok(())
        }
    }

    /// Extract period wakeup flag from a configuration space.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// dbg!(hw.period_wakeup()?);
    /// # Ok(()) }
    /// ```
    pub fn period_wakeup(&mut self) -> Result<c::c_uint> {
        unsafe {
            let mut period_wakeup = mem::MaybeUninit::uninit();
            errno!(alsa::snd_pcm_hw_params_get_period_wakeup(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                period_wakeup.as_mut_ptr()
            ))?;
            Ok(period_wakeup.assume_init())
        }
    }

    /// Verify if a period time is available inside a configuration space for a PCM.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// dbg!(hw.test_period_time(1000, alsa::Direction::Nearest)?);
    /// # Ok(()) }
    /// ```
    pub fn test_period_time(&mut self, period_time: c::c_uint, dir: Direction) -> Result<bool> {
        unsafe {
            let result = errno!(alsa::snd_pcm_hw_params_test_period_time(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                period_time,
                dir as i32,
            ))?;
            Ok(result == 0)
        }
    }

    /// Restrict a configuration space to contain only one period time.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_period_time(1000, alsa::Direction::Nearest)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_period_time(&mut self, period_time: c::c_uint, dir: Direction) -> Result<()> {
        unsafe {
            errno!(alsa::snd_pcm_hw_params_set_period_time(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                period_time,
                dir as i32,
            ))?;
            Ok(())
        }
    }

    /// Restrict a configuration space with a minimum period time.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_period_time_min(1000, alsa::Direction::Nearest)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_period_time_min(
        &mut self,
        mut period_time: c::c_uint,
        dir: Direction,
    ) -> Result<(c::c_uint, Direction)> {
        unsafe {
            let mut dir = dir as i32;
            errno!(alsa::snd_pcm_hw_params_set_period_time_min(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                &mut period_time,
                &mut dir
            ))?;
            let dir = Direction::from_value(dir);
            Ok((period_time, dir))
        }
    }

    /// Restrict a configuration space with a maximum period time.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_period_time_max(1000, alsa::Direction::Nearest)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_period_time_max(
        &mut self,
        mut period_time: c::c_uint,
        dir: Direction,
    ) -> Result<(c::c_uint, Direction)> {
        unsafe {
            let mut dir = dir as i32;
            errno!(alsa::snd_pcm_hw_params_set_period_time_max(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                &mut period_time,
                &mut dir
            ))?;
            let dir = Direction::from_value(dir);
            Ok((period_time, dir))
        }
    }

    /// Restrict a configuration space to have period times in a given range.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_period_time_minmax(1000, alsa::Direction::Nearest, 10000, alsa::Direction::Nearest)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_period_time_minmax(
        &mut self,
        mut period_time_max: c::c_uint,
        dir_max: Direction,
        mut period_time_min: c::c_uint,
        dir_min: Direction,
    ) -> Result<(c::c_uint, Direction, c::c_uint, Direction)> {
        unsafe {
            let mut dir_min = dir_min as i32;
            let mut dir_max = dir_max as i32;
            errno!(alsa::snd_pcm_hw_params_set_period_time_minmax(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                &mut period_time_min,
                &mut dir_min,
                &mut period_time_max,
                &mut dir_max
            ))?;
            let dir_min = Direction::from_value(dir_min);
            let dir_max = Direction::from_value(dir_max);
            Ok((period_time_min, dir_min, period_time_max, dir_max))
        }
    }

    /// Restrict a configuration space to have period time nearest to a target.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_period_time_near(1000, alsa::Direction::Nearest)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_period_time_near(
        &mut self,
        mut period_time: c::c_uint,
        dir: Direction,
    ) -> Result<(c::c_uint, Direction)> {
        unsafe {
            let mut dir = dir as i32;
            errno!(alsa::snd_pcm_hw_params_set_period_time_near(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                &mut period_time,
                &mut dir
            ))?;
            let dir = Direction::from_value(dir);
            Ok((period_time, dir))
        }
    }

    /// Restrict a configuration space to contain only its minimum period time.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_period_time_first()?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_period_time_first(&mut self) -> Result<(c::c_uint, Direction)> {
        unsafe {
            let mut period_time = mem::MaybeUninit::uninit();
            let mut dir = mem::MaybeUninit::uninit();
            errno!(alsa::snd_pcm_hw_params_set_period_time_first(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                period_time.as_mut_ptr(),
                dir.as_mut_ptr()
            ))?;
            let period_time = period_time.assume_init();
            let dir = Direction::from_value(dir.assume_init());
            Ok((period_time, dir))
        }
    }

    /// Restrict a configuration space to contain only its maximum period time.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_period_time_last()?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_period_time_last(&mut self) -> Result<(c::c_uint, Direction)> {
        unsafe {
            let mut period_time = mem::MaybeUninit::uninit();
            let mut dir = mem::MaybeUninit::uninit();
            errno!(alsa::snd_pcm_hw_params_set_period_time_last(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                period_time.as_mut_ptr(),
                dir.as_mut_ptr()
            ))?;
            let period_time = period_time.assume_init();
            let dir = Direction::from_value(dir.assume_init());
            Ok((period_time, dir))
        }
    }

    /// Verify if a period size is available inside a configuration space for a PCM.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// if hw.test_period_size(128, alsa::Direction::Nearest)? {
    ///     println!("period size supported!");
    /// }
    /// # Ok(()) }
    /// ```
    pub fn test_period_size(&mut self, frames: c::c_ulong, dir: Direction) -> Result<bool> {
        unsafe {
            let result = alsa::snd_pcm_hw_params_test_period_size(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                frames,
                dir as i32,
            );
            Ok(result == 1)
        }
    }

    /// Restrict a configuration space to contain only one period size.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_period_size(128, alsa::Direction::Nearest)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_period_size(&mut self, frames: c::c_ulong, dir: Direction) -> Result<()> {
        unsafe {
            errno!(alsa::snd_pcm_hw_params_set_period_size(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                frames,
                dir as i32,
            ))?;
            Ok(())
        }
    }

    /// Restrict a configuration space with a minimum period size.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_period_size_min(128, alsa::Direction::Nearest)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_period_size_min(
        &mut self,
        mut frames: c::c_ulong,
        dir: Direction,
    ) -> Result<(c::c_ulong, Direction)> {
        unsafe {
            let mut dir = dir as i32;
            errno!(alsa::snd_pcm_hw_params_set_period_size_min(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                &mut frames,
                &mut dir
            ))?;
            let dir = Direction::from_value(dir);
            Ok((frames, dir))
        }
    }

    /// Restrict a configuration space with a maximum period size.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_period_size_max(128, alsa::Direction::Nearest)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_period_size_max(
        &mut self,
        mut frames: c::c_ulong,
        dir: Direction,
    ) -> Result<(c::c_ulong, Direction)> {
        unsafe {
            let mut dir = dir as i32;
            errno!(alsa::snd_pcm_hw_params_set_period_size_max(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                &mut frames,
                &mut dir
            ))?;
            let dir = Direction::from_value(dir);
            Ok((frames, dir))
        }
    }

    /// Restrict a configuration space to have period sizes in a given range.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_period_size_minmax(128, alsa::Direction::Nearest, 256, alsa::Direction::Nearest)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_period_size_minmax(
        &mut self,
        mut frames_min: c::c_ulong,
        dir_min: Direction,
        mut frames_max: c::c_ulong,
        dir_max: Direction,
    ) -> Result<(c::c_ulong, Direction, c::c_ulong, Direction)> {
        unsafe {
            let mut dir_min = dir_min as i32;
            let mut dir_max = dir_max as i32;
            errno!(alsa::snd_pcm_hw_params_set_period_size_minmax(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                &mut frames_min,
                &mut dir_min,
                &mut frames_max,
                &mut dir_max,
            ))?;
            let dir_min = Direction::from_value(dir_min);
            let dir_max = Direction::from_value(dir_max);
            Ok((frames_min, dir_min, frames_max, dir_max))
        }
    }

    /// Restrict a configuration space to have period size nearest to a target.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_period_size_near(1024, alsa::Direction::Nearest)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_period_size_near(
        &mut self,
        mut frames: c::c_ulong,
        dir: Direction,
    ) -> Result<(c::c_ulong, Direction)> {
        unsafe {
            let mut dir = dir as i32;
            errno!(alsa::snd_pcm_hw_params_set_period_size_near(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                &mut frames,
                &mut dir,
            ))?;
            let dir = Direction::from_value(dir);
            Ok((frames, dir))
        }
    }

    /// Restrict a configuration space to contain only its minimum period size.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_period_size_first()?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_period_size_first(&mut self) -> Result<(c::c_ulong, Direction)> {
        unsafe {
            let mut frames = mem::MaybeUninit::uninit();
            let mut dir = mem::MaybeUninit::uninit();
            errno!(alsa::snd_pcm_hw_params_set_period_size_first(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                frames.as_mut_ptr(),
                dir.as_mut_ptr()
            ))?;
            let frames = frames.assume_init();
            let dir = Direction::from_value(dir.assume_init());
            Ok((frames, dir))
        }
    }

    /// Restrict a configuration space to contain only its maximum period size.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_period_size_last()?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_period_size_last(&mut self) -> Result<(c::c_ulong, Direction)> {
        unsafe {
            let mut frames = mem::MaybeUninit::uninit();
            let mut dir = mem::MaybeUninit::uninit();
            errno!(alsa::snd_pcm_hw_params_set_period_size_last(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                frames.as_mut_ptr(),
                dir.as_mut_ptr()
            ))?;
            let frames = frames.assume_init();
            let dir = Direction::from_value(dir.assume_init());
            Ok((frames, dir))
        }
    }

    /// Restrict a configuration space to contain only integer period sizes.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_period_size_integer()?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_period_size_integer(&mut self) -> Result<()> {
        unsafe {
            errno!(alsa::snd_pcm_hw_params_set_period_size_integer(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
            ))?;
            Ok(())
        }
    }

    /// Verify if a periods count is available inside a configuration space for a PCM.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// if hw.test_periods(128, alsa::Direction::Nearest)? {
    ///     println!("period size supported!");
    /// }
    /// # Ok(()) }
    /// ```
    pub fn test_periods(&mut self, periods: c::c_uint, dir: Direction) -> Result<bool> {
        unsafe {
            let result = alsa::snd_pcm_hw_params_test_periods(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                periods,
                dir as i32,
            );
            Ok(result == 1)
        }
    }

    /// Restrict a configuration space to contain only one periods count.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_periods(128, alsa::Direction::Nearest)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_periods(&mut self, periods: c::c_uint, dir: Direction) -> Result<()> {
        unsafe {
            errno!(alsa::snd_pcm_hw_params_set_periods(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                periods,
                dir as i32,
            ))?;
            Ok(())
        }
    }

    /// Restrict a configuration space with a minimum periods count.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_periods_min(128, alsa::Direction::Nearest)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_periods_min(
        &mut self,
        mut periods: c::c_uint,
        dir: Direction,
    ) -> Result<(c::c_uint, Direction)> {
        unsafe {
            let mut dir = dir as i32;
            errno!(alsa::snd_pcm_hw_params_set_periods_min(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                &mut periods,
                &mut dir
            ))?;
            let dir = Direction::from_value(dir);
            Ok((periods, dir))
        }
    }

    /// Restrict a configuration space with a maximum periods count.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_periods_max(128, alsa::Direction::Nearest)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_periods_max(
        &mut self,
        mut periods: c::c_uint,
        dir: Direction,
    ) -> Result<(c::c_uint, Direction)> {
        unsafe {
            let mut dir = dir as i32;
            errno!(alsa::snd_pcm_hw_params_set_periods_max(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                &mut periods,
                &mut dir
            ))?;
            let dir = Direction::from_value(dir);
            Ok((periods, dir))
        }
    }

    /// Restrict a configuration space to have periods counts in a given range.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_periods_minmax(128, alsa::Direction::Nearest, 256, alsa::Direction::Nearest)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_periods_minmax(
        &mut self,
        mut periods_min: c::c_uint,
        dir_min: Direction,
        mut periods_max: c::c_uint,
        dir_max: Direction,
    ) -> Result<(c::c_uint, Direction, c::c_uint, Direction)> {
        unsafe {
            let mut dir_min = dir_min as i32;
            let mut dir_max = dir_max as i32;
            errno!(alsa::snd_pcm_hw_params_set_periods_minmax(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                &mut periods_min,
                &mut dir_min,
                &mut periods_max,
                &mut dir_max,
            ))?;
            let dir_min = Direction::from_value(dir_min);
            let dir_max = Direction::from_value(dir_max);
            Ok((periods_min, dir_min, periods_max, dir_max))
        }
    }

    /// Restrict a configuration space to have periods count nearest to a target.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_periods_near(1024, alsa::Direction::Nearest)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_periods_near(
        &mut self,
        mut periods: c::c_uint,
        dir: Direction,
    ) -> Result<(c::c_uint, Direction)> {
        unsafe {
            let mut dir = dir as i32;
            errno!(alsa::snd_pcm_hw_params_set_periods_near(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                &mut periods,
                &mut dir,
            ))?;
            let dir = Direction::from_value(dir);
            Ok((periods, dir))
        }
    }

    /// Restrict a configuration space to contain only its minimum periods count.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_periods_first()?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_periods_first(&mut self) -> Result<(c::c_uint, Direction)> {
        unsafe {
            let mut periods = mem::MaybeUninit::uninit();
            let mut dir = mem::MaybeUninit::uninit();
            errno!(alsa::snd_pcm_hw_params_set_periods_first(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                periods.as_mut_ptr(),
                dir.as_mut_ptr()
            ))?;
            let periods = periods.assume_init();
            let dir = Direction::from_value(dir.assume_init());
            Ok((periods, dir))
        }
    }

    /// Restrict a configuration space to contain only its maximum periods count.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_periods_last()?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_periods_last(&mut self) -> Result<(c::c_uint, Direction)> {
        unsafe {
            let mut periods = mem::MaybeUninit::uninit();
            let mut dir = mem::MaybeUninit::uninit();
            errno!(alsa::snd_pcm_hw_params_set_periods_last(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                periods.as_mut_ptr(),
                dir.as_mut_ptr()
            ))?;
            let periods = periods.assume_init();
            let dir = Direction::from_value(dir.assume_init());
            Ok((periods, dir))
        }
    }

    /// Restrict a configuration space to contain only integer periods counts.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_periods_integer()?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_periods_integer(&mut self) -> Result<()> {
        unsafe {
            errno!(alsa::snd_pcm_hw_params_set_periods_integer(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
            ))?;
            Ok(())
        }
    }

    /// Verify if a buffer time is available inside a configuration space for a PCM.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// dbg!(hw.test_buffer_time(10_000, alsa::Direction::Nearest)?);
    /// # Ok(()) }
    /// ```
    pub fn test_buffer_time(&mut self, buffer_time: c::c_uint, dir: Direction) -> Result<bool> {
        unsafe {
            let result = errno!(alsa::snd_pcm_hw_params_test_buffer_time(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                buffer_time,
                dir as i32,
            ))?;
            Ok(result == 0)
        }
    }

    /// Restrict a configuration space to contain only one buffer time.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_buffer_time(10_000, alsa::Direction::Nearest)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_buffer_time(&mut self, buffer_time: c::c_uint, dir: Direction) -> Result<()> {
        unsafe {
            errno!(alsa::snd_pcm_hw_params_set_buffer_time(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                buffer_time,
                dir as i32,
            ))?;
            Ok(())
        }
    }

    /// Restrict a configuration space with a minimum buffer time.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_buffer_time_min(10_000, alsa::Direction::Nearest)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_buffer_time_min(
        &mut self,
        mut buffer_time: c::c_uint,
        dir: Direction,
    ) -> Result<(c::c_uint, Direction)> {
        unsafe {
            let mut dir = dir as i32;
            errno!(alsa::snd_pcm_hw_params_set_buffer_time_min(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                &mut buffer_time,
                &mut dir
            ))?;
            let dir = Direction::from_value(dir);
            Ok((buffer_time, dir))
        }
    }

    /// Restrict a configuration space with a maximum buffer time.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_buffer_time_max(10_000, alsa::Direction::Nearest)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_buffer_time_max(
        &mut self,
        mut buffer_time: c::c_uint,
        dir: Direction,
    ) -> Result<(c::c_uint, Direction)> {
        unsafe {
            let mut dir = dir as i32;
            errno!(alsa::snd_pcm_hw_params_set_buffer_time_max(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                &mut buffer_time,
                &mut dir
            ))?;
            let dir = Direction::from_value(dir);
            Ok((buffer_time, dir))
        }
    }

    /// Restrict a configuration space to have buffer times in a given range.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_buffer_time_minmax(10_000, alsa::Direction::Nearest, 20_000, alsa::Direction::Nearest)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_buffer_time_minmax(
        &mut self,
        mut buffer_time_min: c::c_uint,
        dir_min: Direction,
        mut buffer_time_max: c::c_uint,
        dir_max: Direction,
    ) -> Result<(c::c_uint, Direction, c::c_uint, Direction)> {
        unsafe {
            let mut dir_min = dir_min as i32;
            let mut dir_max = dir_max as i32;
            errno!(alsa::snd_pcm_hw_params_set_buffer_time_minmax(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                &mut buffer_time_min,
                &mut dir_min,
                &mut buffer_time_max,
                &mut dir_max
            ))?;
            let dir_min = Direction::from_value(dir_min);
            let dir_max = Direction::from_value(dir_max);
            Ok((buffer_time_min, dir_min, buffer_time_max, dir_max))
        }
    }

    /// Restrict a configuration space to have buffer time nearest to a target.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_buffer_time_near(10_000, alsa::Direction::Nearest)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_buffer_time_near(
        &mut self,
        mut buffer_time: c::c_uint,
        dir: Direction,
    ) -> Result<(c::c_uint, Direction)> {
        unsafe {
            let mut dir = dir as i32;
            errno!(alsa::snd_pcm_hw_params_set_buffer_time_near(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                &mut buffer_time,
                &mut dir
            ))?;
            let dir = Direction::from_value(dir);
            Ok((buffer_time, dir))
        }
    }

    /// Restrict a configuration space to contain only its minimum buffer time.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_buffer_time_first()?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_buffer_time_first(&mut self) -> Result<(c::c_uint, Direction)> {
        unsafe {
            let mut buffer_time = mem::MaybeUninit::uninit();
            let mut dir = mem::MaybeUninit::uninit();
            errno!(alsa::snd_pcm_hw_params_set_buffer_time_first(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                buffer_time.as_mut_ptr(),
                dir.as_mut_ptr()
            ))?;
            let buffer_time = buffer_time.assume_init();
            let dir = Direction::from_value(dir.assume_init());
            Ok((buffer_time, dir))
        }
    }

    /// Restrict a configuration space to contain only its maximum buffered time.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_buffer_time_last()?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_buffer_time_last(&mut self) -> Result<(c::c_uint, Direction)> {
        unsafe {
            let mut buffer_time = mem::MaybeUninit::uninit();
            let mut dir = mem::MaybeUninit::uninit();
            errno!(alsa::snd_pcm_hw_params_set_buffer_time_last(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                buffer_time.as_mut_ptr(),
                dir.as_mut_ptr()
            ))?;
            let buffer_time = buffer_time.assume_init();
            let dir = Direction::from_value(dir.assume_init());
            Ok((buffer_time, dir))
        }
    }

    /// Verify if a buffer size is available inside a configuration space for a PCM.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// dbg!(hw.test_buffer_size(1024)?);
    /// # Ok(()) }
    /// ```
    pub fn test_buffer_size(&mut self, buffer_size: c::c_ulong) -> Result<bool> {
        unsafe {
            let result = errno!(alsa::snd_pcm_hw_params_test_buffer_size(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                buffer_size
            ))?;
            Ok(result == 0)
        }
    }

    /// Restrict a configuration space to contain only one buffer size.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_buffer_size(1024)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_buffer_size(&mut self, buffer_size: c::c_ulong) -> Result<()> {
        unsafe {
            errno!(alsa::snd_pcm_hw_params_set_buffer_size(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                buffer_size
            ))?;
            Ok(())
        }
    }

    /// Restrict a configuration space with a minimum buffer size.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_buffer_size_min(1024)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_buffer_size_min(&mut self, mut buffer_size: c::c_ulong) -> Result<c::c_ulong> {
        unsafe {
            errno!(alsa::snd_pcm_hw_params_set_buffer_size_min(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                &mut buffer_size
            ))?;
            Ok(buffer_size)
        }
    }

    /// Restrict a configuration space with a maximum buffer size.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_buffer_size_max(1024)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_buffer_size_max(&mut self, mut buffer_size: c::c_ulong) -> Result<c::c_ulong> {
        unsafe {
            errno!(alsa::snd_pcm_hw_params_set_buffer_size_max(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                &mut buffer_size
            ))?;
            Ok(buffer_size)
        }
    }

    /// Restrict a configuration space to have buffer sizes in a given range.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_buffer_size_minmax(1024, 4096)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_buffer_size_minmax(
        &mut self,
        mut buffer_size_min: c::c_ulong,
        mut buffer_size_max: c::c_ulong,
    ) -> Result<(c::c_ulong, c::c_ulong)> {
        unsafe {
            errno!(alsa::snd_pcm_hw_params_set_buffer_size_minmax(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                &mut buffer_size_min,
                &mut buffer_size_max,
            ))?;
            Ok((buffer_size_min, buffer_size_max))
        }
    }

    /// Restrict a configuration space to have buffer size nearest to a target.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_buffer_size_near(1024)?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_buffer_size_near(&mut self, mut buffer_size: c::c_ulong) -> Result<c::c_ulong> {
        unsafe {
            errno!(alsa::snd_pcm_hw_params_set_buffer_size_near(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                &mut buffer_size
            ))?;
            Ok(buffer_size)
        }
    }

    /// Restrict a configuration space to contain only its minimum buffer size.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_buffer_size_first()?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_buffer_size_first(&mut self) -> Result<c::c_ulong> {
        unsafe {
            let mut buffer_size = mem::MaybeUninit::uninit();
            errno!(alsa::snd_pcm_hw_params_set_buffer_size_first(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                buffer_size.as_mut_ptr()
            ))?;
            Ok(buffer_size.assume_init())
        }
    }

    /// Restrict a configuration space to contain only its maximum buffer size.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    /// let mut hw = pcm.hardware_parameters_any()?;
    ///
    /// let actual = hw.set_buffer_size_last()?;
    /// dbg!(actual);
    /// # Ok(()) }
    /// ```
    pub fn set_buffer_size_last(&mut self) -> Result<c::c_ulong> {
        unsafe {
            let mut buffer_size = mem::MaybeUninit::uninit();
            errno!(alsa::snd_pcm_hw_params_set_buffer_size_last(
                self.pcm.as_mut(),
                self.base.handle.as_mut(),
                buffer_size.as_mut_ptr()
            ))?;
            Ok(buffer_size.assume_init())
        }
    }

    // Note: subformat related things is not implemented.

    // int snd_pcm_hw_params_get_subformat (const snd_pcm_hw_params_t *params, snd_pcm_subformat_t *subformat)
    // Extract subformat from a configuration space.

    // int snd_pcm_hw_params_test_subformat (snd_pcm_t *pcm, snd_pcm_hw_params_t *params, snd_pcm_subformat_t subformat)
    // Verify if a subformat is available inside a configuration space for a PCM.

    // int snd_pcm_hw_params_set_subformat (snd_pcm_t *pcm, snd_pcm_hw_params_t *params, snd_pcm_subformat_t subformat)
    // Restrict a configuration space to contain only one subformat.

    // int snd_pcm_hw_params_set_subformat_first (snd_pcm_t *pcm, snd_pcm_hw_params_t *params, snd_pcm_subformat_t *subformat)
    // Restrict a configuration space to contain only its first subformat.

    // int snd_pcm_hw_params_set_subformat_last (snd_pcm_t *pcm, snd_pcm_hw_params_t *params, snd_pcm_subformat_t *subformat)
    // Restrict a configuration space to contain only its last subformat.

    // int snd_pcm_hw_params_set_subformat_mask (snd_pcm_t *pcm, snd_pcm_hw_params_t *params, snd_pcm_subformat_mask_t *mask)
    // Restrict a configuration space to contain only a set of subformats.

    // void snd_pcm_hw_params_get_subformat_mask (snd_pcm_hw_params_t *params, snd_pcm_subformat_mask_t *mask)
    // Get subformat mask from a configuration space.
}

impl ops::Deref for HardwareParametersMut<'_> {
    type Target = HardwareParameters;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
