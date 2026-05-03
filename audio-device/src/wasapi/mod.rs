//! An idiomatic Rust WASAPI interface.

use windows::Win32::Media::Audio::{
    IAudioClient, IMMDeviceEnumerator, MMDeviceEnumerator, eConsole, eRender,
};
use windows::Win32::System::Com::{
    CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx,
};

mod error;
use self::error::ErrorKind;
pub use self::error::{Error, Result};

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

/// The audio prelude to use for wasapi.
pub fn audio_prelude() {
    unsafe {
        let result = CoInitializeEx(None, COINIT_MULTITHREADED);

        if result.is_err() {
            panic!("failed to initialize multithreaded apartment: {result}");
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

    unsafe {
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                .map_err(ErrorKind::CreateInstance)?;

        let Ok(device) = enumerator.GetDefaultAudioEndpoint(eRender, eConsole) else {
            return Ok(None);
        };

        let audio_client: IAudioClient = device
            .Activate(CLSCTX_ALL, None)
            .map_err(ErrorKind::Activate)?;
        tracing::trace!("got audio client");
        Ok(Some(Client { tag, audio_client }))
    }
}
