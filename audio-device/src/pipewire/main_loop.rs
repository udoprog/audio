use core::ptr::{NonNull, null};

use pipewire_sys as pw;

/// A PipeWire main loop.
///
/// See [MainLoop::new].
pub struct MainLoop {
    handle: NonNull<pw::pw_main_loop>,
}

impl MainLoop {
    /// Construct a new main loop object.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use audio_device::pipewire;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let m = pipewire::MainLoop::new();
    /// # Ok(()) }
    /// ```
    pub fn new() -> Self {
        unsafe {
            let handle = NonNull::new_unchecked(pw::pw_main_loop_new(null()));
            Self { handle }
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
