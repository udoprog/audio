use pipewire_sys as pw;
use std::ptr;

/// A PipeWire main loop.
///
/// See [MainLoop::new].
pub struct MainLoop {
    handle: ptr::NonNull<pw::pw_main_loop>,
}

impl MainLoop {
    /// Construct a new main loop object.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use audio_device::pipewire;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let m = pipewire::MainLoop::new();
    /// # Ok(()) }
    /// ```
    pub fn new() -> Self {
        unsafe {
            let mut handle = ptr::NonNull::new_unchecked(pw::pw_main_loop_new(ptr::null()));

            Self {
                handle,
            }
        }
    }
}

impl Drop for MainLoop {
    fn drop(&mut self) {
        unsafe {
            pw::pw_main_loop_destroy(self.handle.as_mut());
        }
    }
}
