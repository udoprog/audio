//! An idiomatic Rust WASAPI interface.

use std::ptr;

use thiserror::Error;
use windows::Win32::System::Com as com;
use windows::Win32::Media::Audio as audio;

mod initialized_client;
pub use self::initialized_client::InitializedClient;

mod client;
pub use self::client::Client;

mod render_client;
pub use self::render_client::RenderClient;

mod buffer_mut;
pub use self::buffer_mut::BufferMut;

mod sample;
pub use self::sample::Sample;

/// WASAPI-specific errors.
#[derive(Debug, Error)]
pub enum Error {
    /// A system error.
    #[error("system error: {0}")]
    Sys(
        #[from]
        #[source]
        windows::core::Error,
    ),
    /// Trying to use a mix format which is not supported by the device.
    #[error("Device doesn't support a compatible mix format")]
    UnsupportedMixFormat,
}

/// The audio prelude to use for wasapi.
pub fn audio_prelude() {
    unsafe {
        if let Err(e) = com::CoInitializeEx(ptr::null_mut(), com::COINIT_MULTITHREADED) {
            panic!("failed to initialize multithreaded apartment: {}", e);
        }
    }
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
    _tag: ste::Tag,
    /// The number of channels in use.
    pub channels: u16,
    /// The sample rate in use.
    pub sample_rate: u32,
    /// The sample format in use.
    pub sample_format: SampleFormat,
}

/// Open the default output device for WASAPI.
#[tracing::instrument(skip_all)]
pub fn default_output_client() -> Result<Option<Client>, Error> {
    let tag = ste::Tag::current_thread();

    let enumerator: audio::IMMDeviceEnumerator = unsafe {
        com::CoCreateInstance(&audio::MMDeviceEnumerator, None, com::CLSCTX_ALL)?
    };

    unsafe {
        let device = enumerator
            .GetDefaultAudioEndpoint(audio::eRender, audio::eConsole);

        let device = match device {
            Ok(device) => device,
            Err(..) => return Ok(None),
        };

        tracing::trace!("got default audio endpoint");
        let audio_client: audio::IAudioClient = device.Activate(com::CLSCTX_ALL, None)?;
        tracing::trace!("got audio client");
        Ok(Some(Client { tag, audio_client }))
    }
}
