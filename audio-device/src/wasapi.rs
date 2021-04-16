//! An idiomatic Rust WASAPI interface.

use std::mem;
use std::ptr;
use thiserror::Error;
use windows::Interface as _;
use windows_sys::Windows::Win32::Com as com;
use windows_sys::Windows::Win32::CoreAudio as core;

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
        windows::Error,
    ),
    /// Trying to use a mix format which is not supported by the device.
    #[error("Device doesn't support a compatible mix format")]
    UnsupportedMixFormat,
}

/// The audio prelude to use for wasapi.
pub fn audio_prelude() {
    if let Err(e) = windows::initialize_mta() {
        panic!("failed to initialize multithreaded apartment: {}", e);
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
    tag: ste::Tag,
    /// The number of channels in use.
    pub channels: u16,
    /// The sample rate in use.
    pub sample_rate: u32,
    /// The sample format in use.
    pub sample_format: SampleFormat,
}

/// Open the default output device for WASAPI.
pub fn default_output_client() -> Result<Option<Client>, Error> {
    let tag = ste::Tag::current_thread();

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
                com::CLSCTX::CLSCTX_ALL.0,
                ptr::null_mut(),
                audio_client.as_mut_ptr() as *mut _,
            )
            .ok()?;

        let audio_client = audio_client.assume_init();

        Ok(Some(Client { tag, audio_client }))
    }
}
