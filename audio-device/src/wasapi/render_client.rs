use crate::wasapi::{BufferMut, Error};
use crate::windows::{AsyncEvent, Event, RawEvent};
use bindings::Windows::Win32::CoreAudio as core;
use bindings::Windows::Win32::SystemServices as ss;
use bindings::Windows::Win32::WindowsProgramming as wp;
use std::marker;
use std::mem;
use std::sync::Arc;

pub struct RenderClient<T, E> {
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
            let mut padding = mem::MaybeUninit::uninit();
            self.audio_client
                .GetCurrentPadding(padding.as_mut_ptr())
                .ok()?;
            Ok(padding.assume_init())
        }
    }

    /// Get the buffer associated with the render client.
    fn get_buffer(&self, frames: u32) -> Result<*mut T, Error> {
        unsafe {
            let mut data = mem::MaybeUninit::uninit();

            self.render_client
                .GetBuffer(frames, data.as_mut_ptr())
                .ok()?;

            Ok(data.assume_init() as *mut T)
        }
    }
}

impl<T> RenderClient<T, Event> {
    /// Get access to the raw mutable buffer.
    ///
    /// This will block until it is appropriate to submit a buffer.
    pub fn buffer_mut(&mut self) -> Result<BufferMut<'_, T>, Error> {
        unsafe {
            loop {
                match ss::WaitForSingleObject(self.event.raw_event(), wp::INFINITE) {
                    ss::WAIT_RETURN_CAUSE::WAIT_OBJECT_0 => (),
                    _ => {
                        return Err(Error::EventFailed);
                    }
                }

                let padding = self.get_current_padding()?;
                let frames = self.buffer_size.saturating_sub(padding);

                if frames == 0 {
                    continue;
                }

                let data = self.get_buffer(frames)?;

                return Ok(BufferMut {
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

impl<T> RenderClient<T, AsyncEvent> {
    /// Get access to the raw mutable buffer.
    ///
    /// This will block until it is appropriate to submit a buffer.
    pub async fn buffer_mut_async(&mut self) -> Result<BufferMut<'_, T>, Error> {
        loop {
            self.event.wait().await;

            let padding = self.get_current_padding()?;
            let frames = self.buffer_size.saturating_sub(padding);

            if frames == 0 {
                continue;
            }

            let data = self.get_buffer(frames)?;

            return Ok(BufferMut {
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

unsafe impl<T, E> Send for RenderClient<T, E> {}
