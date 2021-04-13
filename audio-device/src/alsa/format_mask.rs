use crate::alsa::{Format, Result};
use alsa_sys as alsa;
use std::mem;
use std::ptr;

/// Format mask used in combination with hardware parameters.
///
/// # Examples
///
/// ```rust
/// use audio_device::alsa;
///
/// # fn main() -> anyhow::Result<()> {
/// let mut mask = alsa::FormatMask::new()?;
/// assert!(!mask.test(alsa::Format::S8));
/// assert!(mask.is_empty());
///
/// mask.set(alsa::Format::S8);
/// assert!(!mask.is_empty());
/// assert!(mask.test(alsa::Format::S8));
///
/// mask.reset(alsa::Format::S8);
/// assert!(!mask.test(alsa::Format::S8));
/// assert!(mask.is_empty());
/// # Ok(()) }
/// ```
pub struct FormatMask {
    pub(super) handle: ptr::NonNull<alsa::snd_pcm_format_mask_t>,
}

impl FormatMask {
    /// Construct a new empty access mask.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut mask = alsa::FormatMask::new()?;
    /// assert!(!mask.test(alsa::Format::S8));
    /// # Ok(()) }
    /// ```
    pub fn new() -> Result<Self> {
        unsafe {
            let mut mask = Self::allocate()?;
            mask.none();
            Ok(mask)
        }
    }

    /// Allocate a new access mask. The state of it will be uninitialized.
    pub(super) unsafe fn allocate() -> Result<Self> {
        let mut handle = mem::MaybeUninit::uninit();
        errno!(alsa::snd_pcm_format_mask_malloc(handle.as_mut_ptr()))?;
        let handle = ptr::NonNull::new_unchecked(handle.assume_init());
        Ok(Self { handle })
    }

    /// Test if mask is empty.
    ///
    /// See [FormatMask] documentation.
    pub fn is_empty(&self) -> bool {
        unsafe { alsa::snd_pcm_format_mask_empty(self.handle.as_ptr()) == 1 }
    }

    /// Set all bits.
    ///
    /// See [FormatMask] documentation.
    pub fn any(&mut self) {
        unsafe {
            alsa::snd_pcm_format_mask_any(self.handle.as_mut());
        }
    }

    /// Reset all bits.
    ///
    /// See [FormatMask] documentation.
    pub fn none(&mut self) {
        unsafe {
            alsa::snd_pcm_format_mask_none(self.handle.as_mut());
        }
    }

    /// Make a format present.
    ///
    /// See [FormatMask] documentation.
    pub fn set(&mut self, format: Format) {
        unsafe {
            alsa::snd_pcm_format_mask_set(self.handle.as_mut(), format as libc::c_int);
        }
    }

    /// Make a format missing.
    ///
    /// See [FormatMask] documentation.
    pub fn reset(&mut self, format: Format) {
        unsafe {
            alsa::snd_pcm_format_mask_reset(self.handle.as_mut(), format as libc::c_int);
        }
    }

    /// Test the presence of a format.
    ///
    /// See [FormatMask] documentation.
    pub fn test(&mut self, format: Format) -> bool {
        unsafe { alsa::snd_pcm_format_mask_test(self.handle.as_mut(), format as libc::c_int) == 1 }
    }

    /// Copy one mask to another.
    pub fn copy(&mut self, other: &Self) {
        unsafe {
            alsa::snd_pcm_format_mask_copy(self.handle.as_mut(), other.handle.as_ptr());
        }
    }
}

impl Drop for FormatMask {
    fn drop(&mut self) {
        unsafe {
            let _ = alsa::snd_pcm_format_mask_free(self.handle.as_mut());
        }
    }
}
