use core::marker;
use core::mem;
use core::ptr::null_mut;

use windows::Win32::Media::Audio as audio;
use windows::Win32::Media::KernelStreaming as ks;
use windows::Win32::Media::Multimedia as mm;
use windows::Win32::System::Com as com;
use windows::core::Result as WindowsResult;

use crate::loom::sync::Arc;
use crate::wasapi::error::ErrorKind;
use crate::wasapi::{ClientConfig, Error, InitializedClient, Sample, SampleFormat};
use crate::windows::{AsyncEvent, Event, RawEvent};

/// An audio client.
pub struct Client {
    pub(super) tag: ste::Tag,
    pub(super) audio_client: audio::IAudioClient,
}

impl Client {
    /// Get the default client configuration.
    #[tracing::instrument(skip_all)]
    pub fn default_client_config(&self) -> Result<ClientConfig, Error> {
        let tag = ste::Tag::current_thread();
        tracing::trace!(?tag, "get default client config");

        unsafe {
            let mix_format = self
                .audio_client
                .GetMixFormat()
                .map_err(ErrorKind::GetMixFormat)?;

            let bits_per_sample = (*mix_format).wBitsPerSample;

            let sample_format = match (*mix_format).wFormatTag as u32 {
                ks::WAVE_FORMAT_EXTENSIBLE => {
                    debug_assert_eq! {
                        (*mix_format).cbSize as usize,
                        mem::size_of::<audio::WAVEFORMATEXTENSIBLE>() - mem::size_of::<audio::WAVEFORMATEX>()
                    };

                    let mix_format = mix_format as *const audio::WAVEFORMATEXTENSIBLE;

                    if bits_per_sample == 32
                        && matches!((*mix_format).SubFormat, mm::KSDATAFORMAT_SUBTYPE_IEEE_FLOAT)
                    {
                        SampleFormat::F32
                    } else {
                        return Err(Error::from(ErrorKind::UnsupportedMixFormat));
                    }
                }
                audio::WAVE_FORMAT_PCM => {
                    assert!((*mix_format).cbSize == 0);

                    if bits_per_sample == 16 {
                        SampleFormat::I16
                    } else {
                        return Err(Error::from(ErrorKind::UnsupportedMixFormat));
                    }
                }
                _ => {
                    return Err(Error::from(ErrorKind::UnsupportedMixFormat));
                }
            };

            let channels = (*mix_format).nChannels;
            let sample_rate = (*mix_format).nSamplesPerSec;

            tracing::trace!(
                ?tag,
                ?channels,
                ?sample_rate,
                ?sample_format,
                "got client config"
            );

            Ok(ClientConfig {
                _tag: tag,
                channels,
                sample_rate,
                sample_format,
            })
        }
    }

    /// Try to initialize the client with the given configuration.
    #[tracing::instrument(skip_all)]
    pub fn initialize<T>(&self, config: ClientConfig) -> Result<InitializedClient<T, Event>, Error>
    where
        T: Sample,
    {
        self.initialize_inner(config, || Event::new(false, false))
    }

    cfg_events_driver! {
        /// Try to initialize the client with the given configuration.
        ///
        /// # Panics
        ///
        /// Panics if the audio runtime is not available.
        ///
        /// See [Runtime][crate::runtime::Runtime] for more.
        #[tracing::instrument(skip_all)]
        pub fn initialize_async<T>(
            &self,
            config: ClientConfig,
        ) -> Result<InitializedClient<T, AsyncEvent>, Error>
        where
            T: Sample,
        {
            self.initialize_inner(config, || AsyncEvent::new(false))
        }
    }

    /// Try to initialize the client with the given configuration.
    fn initialize_inner<T, E>(
        &self,
        mut config: ClientConfig,
        make_event: impl FnOnce() -> WindowsResult<E>,
    ) -> Result<InitializedClient<T, E>, Error>
    where
        T: Sample,
        E: RawEvent,
    {
        unsafe {
            let mut mix_format = T::mix_format(config);
            let mut closest_match = null_mut();

            let result = self.audio_client.IsFormatSupported(
                audio::AUDCLNT_SHAREMODE_SHARED,
                &mix_format.Format,
                Some(&mut closest_match),
            );

            if result.is_ok() {
                if !closest_match.is_null() {
                    mix_format = *(closest_match as *mut audio::WAVEFORMATEXTENSIBLE);
                    com::CoTaskMemFree(Some(closest_match.cast()));
                }

                result.ok().map_err(ErrorKind::IsFormatSupported)?;
            } else {
                if !T::is_compatible_with(&*closest_match) {
                    return Err(Error::from(ErrorKind::UnsupportedMixFormat));
                }

                mix_format = *(closest_match as *mut audio::WAVEFORMATEXTENSIBLE);
                config.sample_rate = mix_format.Format.nSamplesPerSec;
                config.channels = mix_format.Format.nChannels;
                com::CoTaskMemFree(Some(closest_match.cast()));
            };

            tracing::trace!("initializing audio client");

            self.audio_client
                .Initialize(
                    audio::AUDCLNT_SHAREMODE_SHARED,
                    audio::AUDCLNT_STREAMFLAGS_EVENTCALLBACK,
                    0,
                    0,
                    &mix_format.Format,
                    None,
                )
                .map_err(ErrorKind::Initialize)?;

            let event = Arc::new(make_event().map_err(ErrorKind::MakeEvent)?);

            tracing::trace!("set event handle");

            self.audio_client
                .SetEventHandle(event.raw_event())
                .map_err(ErrorKind::SetEventHandle)?;

            let buffer_size = self
                .audio_client
                .GetBufferSize()
                .map_err(ErrorKind::GetBufferSize)?;

            tracing::trace!(?buffer_size, "initialized client");

            Ok(InitializedClient {
                tag: self.tag,
                audio_client: self.audio_client.clone(),
                config,
                buffer_size,
                event,
                _marker: marker::PhantomData,
            })
        }
    }

    /// Start playback on device.
    pub fn start(&self) -> Result<(), Error> {
        unsafe {
            self.audio_client.Start().map_err(ErrorKind::Start)?;
        }

        Ok(())
    }

    /// Stop playback on device.
    pub fn stop(&self) -> Result<(), Error> {
        unsafe {
            self.audio_client.Stop().map_err(ErrorKind::Stop)?;
        }

        Ok(())
    }
}

// Safety: thread safety is ensured through tagging with ste::Tag.
unsafe impl Send for Client {}
