use crate::loom::sync::Arc;
use crate::wasapi::{BufferMut, Error};
use crate::windows::{Event, RawEvent};
use std::marker;
use windows_sys::Windows::Win32::Media::Audio::CoreAudio as core;
use windows_sys::Windows::Win32::System::Threading as th;
use windows_sys::Windows::Win32::System::WindowsProgramming as wp;

/// A typed render client.
pub struct RenderClient<T, E> {
    pub(super) tag: ste::Tag,
    pub(super) audio_client: core::IAudioClient,
    pub(super) render_client: core::IAudioRenderClient,
    pub(super) buffer_size: u32,
    pub(super) channels: usize,
    pub(super) event: Arc<E>,
    pub(super) _marker: marker::PhantomData<T>,
}

impl<T, E> RenderClient<T, E> {
    fn get_current_padding(&self) -> Result<u32, Error> {
        unsafe {
            let padding = self.audio_client
                .GetCurrentPadding()?;
            Ok(padding)
        }
    }

    /// Get the buffer associated with the render client.
    fn get_buffer(&self, frames: u32) -> Result<*mut T, Error> {
        unsafe {
            let data = self.render_client
                .GetBuffer(frames)?;

            Ok(data as *mut T)
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
                match th::WaitForSingleObject(self.event.raw_event(), wp::INFINITE) {
                    th::WAIT_OBJECT_0 => (),
                    _ => {
                        return Err(Error::from(windows::Error::new(
                            windows::HRESULT::from_thread(),
                            "waiting for event failed",
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
