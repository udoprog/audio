use bindings::Windows::Win32::CoreAudio as core;
use std::mem;
use std::ptr;
use thiserror::Error;
use windows::Interface as _;

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
    pub channels: u16,
    pub sample_rate: u32,
    pub sample_format: SampleFormat,
}

/// Open the default output device for WASAPI.
pub fn default_output_client() -> Result<Option<Client>, Error> {
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
