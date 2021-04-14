use crate::driver::Events;
use crate::wasapi::{ClientConfig, Error, InitializedClient, Sample, SampleFormat};
use crate::windows::{AsyncEvent, Event, RawEvent};
use std::marker;
use std::mem;
use std::ptr;
use std::sync::Arc;
use windows_sys::Windows::Win32::Com as com;
use windows_sys::Windows::Win32::CoreAudio as core;
use windows_sys::Windows::Win32::Multimedia as mm;
use windows_sys::Windows::Win32::SystemServices as ss;

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
            let mut mix_format = mem::MaybeUninit::<*mut mm::WAVEFORMATEX>::zeroed();

            self.audio_client
                .GetMixFormat(mix_format.as_mut_ptr())
                .ok()?;

            let mix_format = mix_format.assume_init() as *const mm::WAVEFORMATEX;

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

    /// Try to initialize the client with the given configuration.
    pub fn initialize_async<T>(
        &self,
        events: &Events,
        config: ClientConfig,
    ) -> Result<InitializedClient<T, AsyncEvent>, Error>
    where
        T: Sample,
    {
        self.initialize_inner(config, || events.event(false))
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
            let mut closest_match: *mut mm::WAVEFORMATEXTENSIBLE = ptr::null_mut();

            let result: windows::HRESULT = self.audio_client.IsFormatSupported(
                core::AUDCLNT_SHAREMODE::AUDCLNT_SHAREMODE_SHARED,
                &mix_format as *const _ as *const mm::WAVEFORMATEX,
                &mut closest_match as *mut _ as *mut *mut mm::WAVEFORMATEX,
            );

            if result == ss::S_FALSE {
                if !T::is_compatible_with(closest_match as *const _) {
                    return Err(Error::UnsupportedMixFormat);
                }

                mix_format = *closest_match;
                config.sample_rate = mix_format.Format.nSamplesPerSec;
                config.channels = mix_format.Format.nChannels;
                com::CoTaskMemFree(closest_match as *mut _);
            } else {
                debug_assert!(closest_match.is_null());
                result.ok()?;
            };

            self.audio_client
                .Initialize(
                    core::AUDCLNT_SHAREMODE::AUDCLNT_SHAREMODE_SHARED,
                    core::AUDCLNT_STREAMFLAGS_EVENTCALLBACK,
                    0,
                    0,
                    &mix_format as *const _ as *const mm::WAVEFORMATEX,
                    ptr::null_mut(),
                )
                .ok()?;

            let event = Arc::new(event()?);

            self.audio_client.SetEventHandle(event.raw_event()).ok()?;

            let mut buffer_size = mem::MaybeUninit::<u32>::uninit();
            self.audio_client
                .GetBufferSize(buffer_size.as_mut_ptr())
                .ok()?;
            let buffer_size = buffer_size.assume_init();

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
            self.audio_client.Start().ok()?;
        }

        Ok(())
    }

    /// Stop playback on device.
    pub fn stop(&self) -> Result<(), Error> {
        unsafe {
            self.audio_client.Stop().ok()?;
        }

        Ok(())
    }
}

// Safety: thread safety is ensured through tagging with ste::Tag.
unsafe impl Send for Client {}
