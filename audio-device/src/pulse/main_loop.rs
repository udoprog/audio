use core::ffi::CStr;
use core::ptr::NonNull;

use alloc::vec::Vec;

use pulse_sys as pulse;

use crate::pulse::Context;

/// A Pulseaudio main loop.
///
/// See [MainLoop::new].
pub struct MainLoop {
    handle: NonNull<pulse::pa_mainloop>,
    api: NonNull<pulse::pa_mainloop_api>,
}

impl MainLoop {
    /// Construct a new main loop object.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use audio_device::pulse;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let m = pulse::MainLoop::new();
    /// # Ok(()) }
    /// ```
    pub fn new() -> Self {
        unsafe {
            let mut handle = NonNull::new_unchecked(pulse::pa_mainloop_new());
            let api = NonNull::new_unchecked(pulse::pa_mainloop_get_api(handle.as_mut()));

            Self { handle, api }
        }
    }

    /// Instantiate a new connection context with an abstract mainloop API and
    /// an application name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use audio_device::pulse;
    /// use std::ffi::CString;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut m = pulse::MainLoop::new();
    /// let ctx = m.context(&CString::new("My Application")?);
    /// # Ok(()) }
    /// ```
    pub fn context(&mut self, name: &CStr) -> Context {
        unsafe {
            let handle = pulse::pa_context_new(self.api.as_mut(), name.as_ptr());
            assert!(!handle.is_null(), "pa_context_new: returned NULL");

            Context {
                handle: NonNull::new_unchecked(handle),
                callbacks: Vec::new(),
            }
        }
    }
}

impl Drop for MainLoop {
    fn drop(&mut self) {
        unsafe {
            pulse::pa_mainloop_free(self.handle.as_mut());
        }
    }
}
