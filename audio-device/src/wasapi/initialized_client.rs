use crate::loom::sync::Arc;
use crate::wasapi::{ClientConfig, Error, RenderClient, Sample};
use std::marker;
use windows::Win32::Media::Audio as audio;

/// A client that has been initialized with the given type `T`.
///
/// The type must implement the [Sample] trait to make sure it's appropriate for
/// use with WASAPI.
pub struct InitializedClient<T, E> {
    pub(super) tag: ste::Tag,
    pub(super) audio_client: audio::IAudioClient,
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
    #[tracing::instrument(skip_all)]
    pub fn render_client(&self) -> Result<RenderClient<T, E>, Error> {
        tracing::trace!("initializing render client");

        self.tag.ensure_on_thread();

        let render_client: audio::IAudioRenderClient = unsafe {
            self.audio_client.GetService()?
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
