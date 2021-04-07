use crate::bindings::Windows::Win32::CoreAudio as core;
use crate::wasapi::{ClientConfig, Error, RenderClient, Sample};
use crate::windows::Event;
use std::marker;
use std::mem;
use std::sync::Arc;
use windows::Interface as _;

pub struct InitializedClient<T> {
    pub(super) audio_client: core::IAudioClient,
    pub(super) config: ClientConfig,
    pub(super) buffer_size: u32,
    pub(super) event: Arc<Event>,
    pub(super) _marker: marker::PhantomData<T>,
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
            event: self.event.clone(),
            _marker: marker::PhantomData,
        })
    }
}