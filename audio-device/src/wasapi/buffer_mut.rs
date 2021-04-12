use crate::wasapi::Error;
use bindings::Windows::Win32::CoreAudio as core;
use std::marker;
use std::ops;
use std::slice;

pub struct BufferMut<'a, T> {
    pub(super) render_client: &'a mut core::IAudioRenderClient,
    pub(super) data: *mut T,
    pub(super) frames_available: u32,
    pub(super) len: usize,
    pub(super) in_use: bool,
    pub(super) _marker: marker::PhantomData<&'a mut [T]>,
}

impl<'a, T> BufferMut<'a, T> {
    /// Release the buffer allowing the audio device to consume it.
    pub fn release(mut self) -> Result<(), Error> {
        if std::mem::take(&mut self.in_use) {
            unsafe {
                self.render_client
                    .ReleaseBuffer(self.frames_available, 0)
                    .ok()?;
            }
        }

        Ok(())
    }
}

impl<'a, T> Drop for BufferMut<'a, T> {
    fn drop(&mut self) {
        if std::mem::take(&mut self.in_use) {
            unsafe {
                self.render_client
                    .ReleaseBuffer(self.frames_available, 0)
                    .ok()
                    .unwrap();
            }
        }
    }
}

impl<'a, T> ops::Deref for BufferMut<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        debug_assert!(self.in_use);
        unsafe { slice::from_raw_parts(self.data, self.len) }
    }
}

impl<'a, T> ops::DerefMut for BufferMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        debug_assert!(self.in_use);
        unsafe { slice::from_raw_parts_mut(self.data, self.len) }
    }
}
