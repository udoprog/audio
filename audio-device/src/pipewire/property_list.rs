use pipewire_sys as pw;
use std::ptr;

/// A property list object.
///
/// See [PropertyList::new].
pub struct PropertyList {
    pub(super) handle: ptr::NonNull<pw::pw_properties>,
}

impl PropertyList {
    /// Construct a property list.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio_device::pipewire;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let props = pipewire::PropertyList::new();
    /// # Ok(()) }
    /// ```
    pub fn new() -> Self {
        unsafe {
            Self {
                handle: ptr::NonNull::new_unchecked(pw::pw_properties_new(ptr::null())),
            }
        }
    }
}

impl Drop for PropertyList {
    fn drop(&mut self) {
        unsafe {
            pw::pw_properties_free(self.handle.as_mut());
        }
    }
}
