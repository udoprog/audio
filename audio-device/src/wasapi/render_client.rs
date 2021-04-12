use crate::wasapi::{BufferMut, Error};
use crate::windows::Event;
use bindings::Windows::Win32::CoreAudio as core;
use bindings::Windows::Win32::SystemServices as ss;
use bindings::Windows::Win32::WindowsProgramming as wp;
use std::marker;
use std::mem;
use std::sync::Arc;

pub struct RenderClient<T> {
    pub(super) audio_client: core::IAudioClient,
    pub(super) render_client: core::IAudioRenderClient,
    pub(super) buffer_size: u32,
    pub(super) channels: usize,
    pub(super) event: Arc<Event>,
    pub(super) _marker: marker::PhantomData<T>,
}

impl<T> RenderClient<T> {
    /// Get access to the raw mutable buffer.
    ///
    /// This will block until it is appropriate to submit a buffer.
    pub fn buffer_mut(&mut self) -> Result<BufferMut<'_, T>, Error> {
        unsafe {
            match ss::WaitForSingleObject(self.event.handle(), wp::INFINITE) {
                ss::WAIT_RETURN_CAUSE::WAIT_OBJECT_0 => (),
                _ => {
                    return Err(Error::EventFailed);
                }
            }

            let padding = self.get_current_padding()?;
            let frames_available = self.buffer_size.saturating_sub(padding);

            debug_assert!(frames_available > 0);

            let mut data = mem::MaybeUninit::uninit();

            self.render_client
                .GetBuffer(frames_available, data.as_mut_ptr())
                .ok()?;

            let data = data.assume_init() as *mut T;

            Ok(BufferMut {
                render_client: &mut self.render_client,
                data: data as *mut T,
                frames_available,
                len: frames_available as usize * self.channels,
                in_use: true,
                _marker: marker::PhantomData,
            })
        }
    }

    fn get_current_padding(&self) -> Result<u32, Error> {
        unsafe {
            let mut padding = mem::MaybeUninit::uninit();
            self.audio_client
                .GetCurrentPadding(padding.as_mut_ptr())
                .ok()?;
            Ok(padding.assume_init())
        }
    }
}
