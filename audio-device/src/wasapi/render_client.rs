use core::marker;

use windows::Win32::Foundation as f;
use windows::Win32::Media::Audio as audio;
use windows::Win32::System::Threading as th;

use crate::loom::sync::Arc;
use crate::wasapi::error::ErrorKind;
use crate::wasapi::{BufferMut, Error};
use crate::windows::{Event, RawEvent};

/// A typed render client.
pub struct RenderClient<T, E> {
    pub(super) tag: ste::Tag,
    pub(super) audio_client: audio::IAudioClient,
    pub(super) render_client: audio::IAudioRenderClient,
    pub(super) buffer_size: u32,
    pub(super) channels: usize,
    pub(super) event: Arc<E>,
    pub(super) _marker: marker::PhantomData<T>,
}

impl<T, E> RenderClient<T, E> {
    fn get_current_padding(&self) -> Result<u32, Error> {
        unsafe {
            let padding = self
                .audio_client
                .GetCurrentPadding()
                .map_err(ErrorKind::GetCurrentPadding)?;

            Ok(padding)
        }
    }

    /// Get the buffer associated with the render client.
    fn get_buffer(&self, frames: u32) -> Result<*mut T, Error> {
        unsafe {
            let data = self
                .render_client
                .GetBuffer(frames)
                .map_err(ErrorKind::GetBuffer)?;

            Ok(data.cast())
        }
    }
}

impl<T> RenderClient<T, Event> {
    /// Get access to the raw mutable buffer.
    ///
    /// This will block until it is appropriate to submit a buffer.
    pub fn buffer_mut(&mut self) -> Result<BufferMut<'_, T>, Error> {
        self.tag.ensure_on_thread();

        unsafe {
            loop {
                match th::WaitForSingleObject(self.event.raw_event(), th::INFINITE) {
                    f::WAIT_OBJECT_0 => (),
                    _ => {
                        return Err(Error::from(ErrorKind::WaitError(
                            windows::core::Error::from_thread(),
                        )));
                    }
                }

                let padding = self.get_current_padding()?;
                let frames = self.buffer_size.saturating_sub(padding);

                if frames == 0 {
                    continue;
                }

                let data = self.get_buffer(frames)?;

                return Ok(BufferMut {
                    tag: self.tag,
                    render_client: &mut self.render_client,
                    data,
                    frames,
                    len: frames as usize * self.channels,
                    in_use: true,
                    _marker: marker::PhantomData,
                });
            }
        }
    }
}

cfg_events_driver! {
    use crate::windows::AsyncEvent;

    impl<T> RenderClient<T, AsyncEvent> {
        /// Get access to the raw mutable buffer.
        ///
        /// This will block until it is appropriate to submit a buffer.
        pub async fn buffer_mut_async(&mut self) -> Result<BufferMut<'_, T>, Error> {
            loop {
                self.event.wait().await;
                self.tag.ensure_on_thread();

                let padding = self.get_current_padding()?;
                let frames = self.buffer_size.saturating_sub(padding);

                if frames == 0 {
                    continue;
                }

                let data = self.get_buffer(frames)?;

                return Ok(BufferMut {
                    tag: self.tag,
                    render_client: &mut self.render_client,
                    data,
                    frames,
                    len: frames as usize * self.channels,
                    in_use: true,
                    _marker: marker::PhantomData,
                });
            }
        }
    }
}

// Safety: thread safety is ensured through tagging with ste::Tag.
unsafe impl<T, E> Send for RenderClient<T, E> {}
