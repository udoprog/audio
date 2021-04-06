use crate::bindings::Windows::Win32::ApplicationInstallationAndServicing as s;
use crate::bindings::Windows::Win32::XAudio2 as x2;
use thiserror::Error;

mod audio;
pub use self::audio::Audio;

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

/// Open the default output device for WASAPI.
pub fn default_audio() -> Result<Option<Audio>, Error> {
    unsafe {
        let mut audio = None;
        let audio = x2::XAudio2CreateWithVersionInfo(&mut audio, 0, x2::Processor1, s::NTDDI_WIN7)
            .and_some(audio)?;
        Ok(Some(Audio { audio }))
    }
}
