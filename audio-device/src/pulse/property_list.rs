use pulse_sys as pulse;
use std::ptr;
use crate::libc as c;

/// A property list object.
///
/// Basically a dictionary with ASCII strings as keys and arbitrary data as values.
/// 
/// See [PropertyList::new].
pub struct PropertyList {
    handle: ptr::NonNull<pulse::pa_proplist>,
}

impl PropertyList {
    /// Construct a property list.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio_device::pulse;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let props = pulse::PropertyList::new();
    /// # Ok(()) }
    /// ```
    pub fn new() -> Self {
        unsafe {
            Self {
                handle: ptr::NonNull::new_unchecked(pulse::pa_proplist_new()),
            }
        }
    }

    /// Return the number of entries in the property list.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio_device::pulse;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let props = pulse::PropertyList::new();
    /// assert_eq!(props.len(), 0);
    /// # Ok(()) }
    /// ```
    pub fn len(&self) -> c::c_uint {
        unsafe {
            pulse::pa_proplist_size(self.handle.as_ref())
        }
    }

    /// Return the number of entries in the property list.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio_device::pulse;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let props = pulse::PropertyList::new();
    /// assert!(props.is_empty());
    /// # Ok(()) }
    /// ```
    pub fn is_empty(&self) -> bool {
        unsafe {
            dbg!(pulse::pa_proplist_isempty(self.handle.as_ref())) == 1
        }
    }
}

impl Drop for PropertyList {
    fn drop(&mut self) {
        unsafe {
            pulse::pa_proplist_free(self.handle.as_ptr());
        }
    }
}
