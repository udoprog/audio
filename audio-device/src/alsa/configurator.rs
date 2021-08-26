use crate::alsa::{Access, Direction, Error, Format, Pcm, Result, Sample};
use crate::libc as c;
use std::marker;

/// Default access to configure.
const DEFAULT_ACCESS: Access = Access::ReadWriteInterleaved;
/// Default latency of 500 us.
const DEFAULT_LATENCY: c::c_uint = 500_000;
/// Default number of channels to use is 2.
const DEFAULT_CHANNELS: c::c_uint = 2;
/// Default sample rate to use.
const DEFAULT_RATE: c::c_uint = 44100;

/// The stream configuration used after the configurator has been successfully installed.
///
/// See [Configurator::install].
#[derive(Debug, Clone, Copy)]
pub struct Config {
    /// The number of channels being used.
    pub channels: c::c_uint,
    /// The configured sample rate being used.
    pub rate: c::c_uint,
    /// The configured buffer time.
    pub buffer_time: c::c_uint,
    /// The configured period time.
    pub period_time: c::c_uint,
    /// The configured period size in frames.
    pub period_size: c::c_ulong,
}

/// A simple [Pcm] stream configuration.
///
/// # Examples
///
/// ```no_run
/// use audio_device::alsa;
///
/// # fn main() -> anyhow::Result<()> {
/// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
///
/// let config = pcm.configure::<f32>()
///     .rate(48000)
///     .channels(5)
///     .install()?;
///
/// dbg!(config);
/// # Ok(()) }
/// ```
pub struct Configurator<'a, T> {
    pcm: &'a mut Pcm,
    access: Access,
    format: Format,
    latency: c::c_uint,
    channels: c::c_uint,
    rate: c::c_uint,
    _marker: marker::PhantomData<T>,
}

impl<'a, T> Configurator<'a, T>
where
    T: Sample,
{
    pub(super) fn new(pcm: &'a mut Pcm) -> Self {
        Self {
            pcm,
            access: DEFAULT_ACCESS,
            format: T::DEFAULT_FORMAT,
            latency: DEFAULT_LATENCY,
            channels: DEFAULT_CHANNELS,
            rate: DEFAULT_RATE,
            _marker: marker::PhantomData,
        }
    }

    /// Configure the stream access to use.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    ///
    /// let config = pcm.configure::<f32>()
    ///     .access(alsa::Access::MmapInterleaved)
    ///     .install()?;
    ///
    /// dbg!(config);
    /// # Ok(()) }
    /// ```
    pub fn access(self, access: Access) -> Self {
        Self { access, ..self }
    }

    /// Configure the stream format to use.
    ///
    /// This will check that the format is appropriate to use by the current
    /// sample. Inappropriate formats will be signalled with
    /// [Error::FormatMismatch].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    ///
    /// let config = pcm.configure::<f32>()
    ///     .format(alsa::Format::FloatLE)?
    ///     .install()?;
    ///
    /// dbg!(config);
    /// # Ok(()) }
    /// ```
    pub fn format(self, format: Format) -> Result<Self> {
        if !T::test(format) {
            return Err(Error::FormatMismatch {
                ty: T::describe(),
                format,
            });
        }

        Ok(Self { format, ..self })
    }

    /// Configure the stream latency to use.
    ///
    /// Will never accept a latency higher than `2**32`. Anything larger will be floored to it.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    ///
    /// let config = pcm.configure::<f32>()
    ///     .latency(std::time::Duration::from_micros(500))
    ///     .install()?;
    ///
    /// dbg!(config);
    /// # Ok(()) }
    /// ```
    pub fn latency(self, latency: std::time::Duration) -> Self {
        let latency = u128::min(u32::MAX as u128, latency.as_micros()) as u32;
        Self { latency, ..self }
    }

    /// Configure the number of channels to use.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    ///
    /// let config = pcm.configure::<f32>()
    ///     .channels(5)
    ///     .install()?;
    ///
    /// dbg!(config);
    /// # Ok(()) }
    /// ```
    pub fn channels(self, channels: c::c_uint) -> Self {
        Self { channels, ..self }
    }

    /// Configure the sample rate to use.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    ///
    /// let config = pcm.configure::<f32>()
    ///     .rate(48000)
    ///     .install()?;
    ///
    /// dbg!(config);
    /// # Ok(()) }
    /// ```
    pub fn rate(self, rate: c::c_uint) -> Self {
        Self { rate, ..self }
    }

    /// Install the current configuration and return the one which is used by
    /// the underlying PCM.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;
    ///
    /// let config = pcm.configure::<f32>()
    ///     .channels(2)
    ///     .rate(48000)
    ///     .install()?;
    ///
    /// dbg!(config);
    /// # Ok(()) }
    /// ```
    pub fn install(self) -> Result<Config> {
        let mut hw = self.pcm.hardware_parameters_any()?;
        hw.set_rate_resample(false)?;
        hw.set_access(self.access)?;
        hw.set_format(self.format)?;
        hw.set_channels(self.channels)?;

        let (rate, _) = hw.set_rate_near(self.rate, Direction::Nearest)?;
        let (buffer_time, _) = hw.set_buffer_time_near(self.latency, Direction::Nearest)?;
        let period_time = self.latency / 4;
        let (period_time, _) = hw.set_period_time_near(period_time, Direction::Nearest)?;
        let buffer_size = hw.buffer_size()?;
        let (period_size, _) = hw.period_size()?;

        hw.install()?;

        let mut sw = self.pcm.software_parameters_mut()?;
        sw.set_start_threshold((buffer_size / period_size) * period_size)?;
        sw.set_available_min(period_size)?;
        sw.install()?;

        Ok(Config {
            channels: self.channels,
            rate,
            buffer_time,
            period_time,
            period_size,
        })
    }
}
