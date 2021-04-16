use crate::loom::sync::Arc;
use crate::wasapi::{ClientConfig, Error, RenderClient, Sample};
use std::marker;
use std::mem;
use windows::Interface as _;
use windows_sys::Windows::Win32::CoreAudio as core;

/// A client that has been initialized with the given type `T`.
///
/// The type must implement the [Sample] trait to make sure it's appropriate for
/// use with WASAPI.
pub struct InitializedClient<T, E> {
    pub(super) tag: ste::Tag,
    pub(super) audio_client: core::IAudioClient,
    pub(super) config: ClientConfig,
    pub(super) buffer_size: u32,
    pub(super) event: Arc<E>,
    pub(super) _marker: marker::PhantomData<T>,
}

impl<T, E> InitializedClient<T, E>
where
    T: Sample,
{
    /// Get the initialized client configuration.
    pub fn config(&self) -> ClientConfig {
        self.config
    }

    /// Construct a render client used for writing output into.
    pub fn render_client(&self) -> Result<RenderClient<T, E>, Error> {
        self.tag.ensure_on_thread();

        let render_client: core::IAudioRenderClient = unsafe {
            let mut render_client = std::ptr::null_mut();

            self.audio_client
                .GetService(&core::IAudioRenderClient::IID, &mut render_client)
                .ok()?;

            debug_assert!(!render_client.is_null());
            mem::transmute(render_client)
        };

        Ok(RenderClient {
            tag: self.tag,
            audio_client: self.audio_client.clone(),
            render_client,
            buffer_size: self.buffer_size,
            channels: self.config.channels as usize,
            event: self.event.clone(),
            _marker: marker::PhantomData,
        })
    }
}

// Safety: thread safety is ensured through tagging with ste::Tag.
unsafe impl<T, E> Send for InitializedClient<T, E> {}
