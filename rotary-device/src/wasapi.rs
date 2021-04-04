use crate::bindings::Windows::Win32::Com as com;
use crate::bindings::Windows::Win32::CoreAudio as core;
use crate::bindings::Windows::Win32::Multimedia as mm;
use crate::bindings::Windows::Win32::SystemServices as ss;
use crate::bindings::Windows::Win32::WindowsProgramming as wp;
use std::marker;
use std::mem;
use std::ops;
use std::ptr;
use std::slice;
use thiserror::Error;
use windows::Interface;

mod sample;
pub use self::sample::Sample;

pub const CLSCTX_INPROC_SERVER: u32 = 0x1;
pub const CLSCTX_INPROC_HANDLER: u32 = 0x2;
pub const CLSCTX_LOCAL_SERVER: u32 = 0x4;
pub const CLSCTX_REMOTE_SERVER: u32 = 0x10;

pub const CLSCTX_ALL: u32 =
    CLSCTX_INPROC_SERVER | CLSCTX_INPROC_HANDLER | CLSCTX_LOCAL_SERVER | CLSCTX_REMOTE_SERVER;

/// WASAPI-specific errors.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Windows error")]
    Io(#[from] windows::Error),
    #[error("Device doesn't support a compatible mix format")]
    UnsupportedMixFormat,
    #[error("Failed to wait for event to clear")]
    EventFailed,
}

/// The sample format detected for the device.
#[derive(Debug, Clone, Copy)]
pub enum SampleFormat {
    /// A 16-bit sample format.
    I16,
    /// A 32-bit floating point sample format.
    F32,
}

/// A client configuration.
///
/// Constructed through [Client::default_client_config].
#[derive(Debug, Clone, Copy)]
pub struct ClientConfig {
    pub channels: u16,
    pub sample_rate: u32,
    pub sample_format: SampleFormat,
}

pub struct BufferMut<'a, T> {
    render_client: &'a mut core::IAudioRenderClient,
    data: *mut T,
    frames_available: u32,
    len: usize,
    in_use: bool,
    _marker: marker::PhantomData<&'a mut [T]>,
}

impl<'a, T> BufferMut<'a, T> {
    /// Release the buffer allowing the audio device to consume it.
    pub fn release(mut self) -> Result<(), Error> {
        if std::mem::take(&mut self.in_use) {
            unsafe {
                self.render_client
                    .ReleaseBuffer(self.frames_available, 0)
                    .ok()?;
            }
        }

        Ok(())
    }
}

impl<'a, T> Drop for BufferMut<'a, T> {
    fn drop(&mut self) {
        if std::mem::take(&mut self.in_use) {
            unsafe {
                self.render_client
                    .ReleaseBuffer(self.frames_available, 0)
                    .ok()
                    .unwrap();
            }
        }
    }
}

impl<'a, T> ops::Deref for BufferMut<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        debug_assert!(self.in_use);
        unsafe { slice::from_raw_parts(self.data, self.len) }
    }
}

impl<'a, T> ops::DerefMut for BufferMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        debug_assert!(self.in_use);
        unsafe { slice::from_raw_parts_mut(self.data, self.len) }
    }
}

pub struct RenderClient<T> {
    audio_client: core::IAudioClient,
    render_client: core::IAudioRenderClient,
    buffer_size: u32,
    channels: usize,
    event: ss::HANDLE,
    _marker: marker::PhantomData<T>,
}

impl<T> RenderClient<T> {
    /// Get access to the raw mutable buffer.
    ///
    /// This will block until it is appropriate to submit a buffer.
    pub fn buffer_mut(&mut self) -> Result<BufferMut<'_, T>, Error> {
        unsafe {
            match ss::WaitForSingleObject(self.event, wp::INFINITE) {
                ss::WAIT_RETURN_CAUSE::WAIT_OBJECT_0 => (),
                _ => {
                    return Err(Error::EventFailed);
                }
            }

            let padding = self.get_current_padding()?;
            let frames_available = self.buffer_size.saturating_sub(padding);

            debug_assert!(frames_available > 0);

            let mut data = mem::MaybeUninit::uninit();

            self.render_client
                .GetBuffer(frames_available, data.as_mut_ptr())
                .ok()?;

            let data = data.assume_init() as *mut T;

            Ok(BufferMut {
                render_client: &mut self.render_client,
                data: data as *mut T,
                frames_available,
                len: frames_available as usize * self.channels,
                in_use: true,
                _marker: marker::PhantomData,
            })
        }
    }

    fn get_current_padding(&self) -> Result<u32, Error> {
        unsafe {
            let mut padding = mem::MaybeUninit::uninit();
            self.audio_client
                .GetCurrentPadding(padding.as_mut_ptr())
                .ok()?;
            Ok(padding.assume_init())
        }
    }
}

pub struct InitializedClient<T> {
    audio_client: core::IAudioClient,
    config: ClientConfig,
    buffer_size: u32,
    event: ss::HANDLE,
    _marker: marker::PhantomData<T>,
}

impl<T> InitializedClient<T>
where
    T: Sample,
{
    /// Get the initialized client configuration.
    pub fn config(&self) -> ClientConfig {
        self.config
    }

    /// Construct a render client used for writing output into.
    pub fn render_client(&self) -> Result<RenderClient<T>, Error> {
        let render_client: core::IAudioRenderClient = unsafe {
            let mut render_client = std::ptr::null_mut();

            self.audio_client
                .GetService(&core::IAudioRenderClient::IID, &mut render_client)
                .ok()?;

            debug_assert!(!render_client.is_null());
            mem::transmute(render_client)
        };

        Ok(RenderClient {
            audio_client: self.audio_client.clone(),
            render_client,
            buffer_size: self.buffer_size,
            channels: self.config.channels as usize,
            event: self.event,
            _marker: marker::PhantomData,
        })
    }
}

/// An audio client.
pub struct Client {
    audio_client: core::IAudioClient,
}

impl Client {
    /// Get the default client configuration.
    pub fn default_client_config(&self) -> Result<ClientConfig, Error> {
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
                channels,
                sample_rate,
                sample_format,
            })
        }
    }

    /// Try to initialize the client with the given configuration.
    pub fn initialize<T>(&self, mut config: ClientConfig) -> Result<InitializedClient<T>, Error>
    where
        T: Sample,
    {
        unsafe {
            let mut mix_format = T::mix_format(config);
            let mut closest_match: *mut mm::WAVEFORMATEXTENSIBLE = ptr::null_mut();

            let result: windows::ErrorCode = self.audio_client.IsFormatSupported(
                core::AUDCLNT_SHAREMODE::AUDCLNT_SHAREMODE_SHARED,
                &mix_format as *const _ as *const mm::WAVEFORMATEX,
                &mut closest_match as *mut _ as *mut *mut mm::WAVEFORMATEX,
            );

            if result == windows::ErrorCode::S_FALSE {
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

            let event = ss::CreateEventA(ptr::null_mut(), false, false, ss::PSTR::default());

            self.audio_client.SetEventHandle(event).ok()?;

            let mut buffer_size = mem::MaybeUninit::<u32>::uninit();
            self.audio_client
                .GetBufferSize(buffer_size.as_mut_ptr())
                .ok()?;
            let buffer_size = buffer_size.assume_init();

            Ok(InitializedClient {
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

/// Open the default input device for WASAPI.
pub fn default_output_device() -> Result<Option<Client>, Error> {
    windows::initialize_mta()?;

    let enumerator: core::IMMDeviceEnumerator =
        windows::create_instance(&core::MMDeviceEnumerator)?;

    let mut device = None;

    unsafe {
        enumerator
            .GetDefaultAudioEndpoint(core::EDataFlow::eRender, core::ERole::eConsole, &mut device)
            .ok()?;

        let device = match device {
            Some(device) => device,
            None => return Ok(None),
        };

        let mut audio_client: mem::MaybeUninit<core::IAudioClient> = mem::MaybeUninit::zeroed();

        device
            .Activate(
                &core::IAudioClient::IID,
                CLSCTX_ALL,
                ptr::null_mut(),
                audio_client.as_mut_ptr() as *mut _,
            )
            .ok()?;

        let audio_client = audio_client.assume_init();

        Ok(Some(Client { audio_client }))
    }
}
