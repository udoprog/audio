use crate::loom::sync::Arc;
use crate::wasapi::{ClientConfig, Error, InitializedClient, Sample, SampleFormat};
use crate::windows::{AsyncEvent, Event, RawEvent};
use std::marker;
use std::mem;
use std::ptr;
use windows_sys::Windows::Win32::System::Com as com;
use windows_sys::Windows::Win32::Media::Audio::CoreAudio as core;
use windows_sys::Windows::Win32::Media::Multimedia as mm;

/// An audio client.
pub struct Client {
    pub(super) tag: ste::Tag,
    pub(super) audio_client: core::IAudioClient,
}

impl Client {
    /// Get the default client configuration.
    pub fn default_client_config(&self) -> Result<ClientConfig, Error> {
        let tag = ste::Tag::current_thread();

        unsafe {
            let mix_format = self.audio_client
                .GetMixFormat()?;

            let bits_per_sample = (*mix_format).wBitsPerSample;

            let sample_format = match (*mix_format).wFormatTag as u32 {
                core::WAVE_FORMAT_EXTENSIBLE => {
                    debug_assert_eq! {
                        (*mix_format).cbSize as usize,
                        mem::size_of::<mm::WAVEFORMATEXTENSIBLE>() - mem::size_of::<mm::WAVEFORMATEX>()
                    };

                    let mix_format = mix_format as *const mm::WAVEFORMATEXTENSIBLE;

                    if bits_per_sample == 32
                        && matches!((*mix_format).SubFormat, mm::KSDATAFORMAT_SUBTYPE_IEEE_FLOAT)
                    {
                        SampleFormat::F32
                    } else {
                        return Err(Error::UnsupportedMixFormat);
                    }
                }
                mm::WAVE_FORMAT_PCM => {
                    assert!((*mix_format).cbSize == 0);

                    if bits_per_sample == 16 {
                        SampleFormat::I16
                    } else {
                        return Err(Error::UnsupportedMixFormat);
                    }
                }
                _ => {
                    return Err(Error::UnsupportedMixFormat);
                }
            };

            let channels = (*mix_format).nChannels;
            let sample_rate = (*mix_format).nSamplesPerSec;

            Ok(ClientConfig {
                tag,
                channels,
                sample_rate,
                sample_format,
            })
        }
    }

    /// Try to initialize the client with the given configuration.
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
    fn initialize_inner<T, F, E>(
        &self,
        mut config: ClientConfig,
        event: F,
    ) -> Result<InitializedClient<T, E>, Error>
    where
        T: Sample,
        F: FnOnce() -> windows::Result<E>,
        E: RawEvent,
    {
        unsafe {
            let mut mix_format = T::mix_format(config);

            let closest_match = self.audio_client.IsFormatSupported(
                core::AUDCLNT_SHAREMODE_SHARED,
                &mix_format as *const _ as *const mm::WAVEFORMATEX,
            )?;

            if result.is_err() {
                if !T::is_compatible_with(closest_match as *const _) {
                    return Err(Error::UnsupportedMixFormat);
                }

                mix_format = *(closest_match as *mut mm::WAVEFORMATEXTENSIBLE);
                config.sample_rate = mix_format.Format.nSamplesPerSec;
                config.channels = mix_format.Format.nChannels;
                com::CoTaskMemFree(closest_match as *mut _);
            } else {
                debug_assert!(closest_match.is_null());
                result.ok()?;
            };

            self.audio_client
                .Initialize(
                    core::AUDCLNT_SHAREMODE_SHARED,
                    core::AUDCLNT_STREAMFLAGS_EVENTCALLBACK,
                    0,
                    0,
                    &mix_format as *const _ as *const mm::WAVEFORMATEX,
                    ptr::null_mut(),
                )?;

            let event = Arc::new(event()?);

            self.audio_client.SetEventHandle(event.raw_event())?;

            let buffer_size = self.audio_client
                .GetBufferSize()?;

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
            self.audio_client.Start()?;
        }

        Ok(())
    }

    /// Stop playback on device.
    pub fn stop(&self) -> Result<(), Error> {
        unsafe {
            self.audio_client.Stop()?;
        }

        Ok(())
    }
}

// Safety: thread safety is ensured through tagging with ste::Tag.
unsafe impl Send for Client {}
