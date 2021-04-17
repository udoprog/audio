use pulse_sys as pulse;
use std::ptr;

/// A Pulseaudio main loop.
///
/// See [MainLoop::new].
pub struct MainLoop {
    handle: ptr::NonNull<pulse::pa_mainloop>,
    api: ptr::NonNull<pulse::pa_mainloop_api>,
}

impl MainLoop {
    /// Construct a new main loop object.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::pulse;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let m = pulse::MainLoop::new();
    /// # Ok(()) }
    /// ```
    pub fn new() -> Self {
        unsafe {
            let mut handle = ptr::NonNull::new_unchecked(pulse::pa_mainloop_new());
            let api = ptr::NonNull::new_unchecked(pulse::pa_mainloop_get_api(handle.as_mut()));

            Self {
                handle,
                api,
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
