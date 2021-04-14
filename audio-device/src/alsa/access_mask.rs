use crate::alsa::{Access, Result};
use crate::libc as c;
use alsa_sys as alsa;
use std::mem;
use std::ptr;

/// Access mask used in combination with hardware parameters.
///
/// # Examples
///
/// ```rust
/// use audio_device::alsa;
///
/// # fn main() -> anyhow::Result<()> {
/// let mut mask = alsa::AccessMask::new()?;
/// assert!(!mask.test(alsa::Access::MmapInterleaved));
/// assert!(mask.is_empty());
///
/// mask.set(alsa::Access::MmapInterleaved);
/// assert!(!mask.is_empty());
/// assert!(mask.test(alsa::Access::MmapInterleaved));
///
/// mask.reset(alsa::Access::MmapInterleaved);
/// assert!(!mask.test(alsa::Access::MmapInterleaved));
/// assert!(mask.is_empty());
/// # Ok(()) }
/// ```
pub struct AccessMask {
    pub(super) handle: ptr::NonNull<alsa::snd_pcm_access_mask_t>,
}

impl AccessMask {
    /// Construct a new empty access mask.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio_device::alsa;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut mask = alsa::AccessMask::new()?;
    /// assert!(!mask.test(alsa::Access::MmapInterleaved));
    /// # Ok(()) }
    /// ```
    pub fn new() -> Result<Self> {
        unsafe {
            let mut mask = Self::allocate()?;
            alsa::snd_pcm_access_mask_none(mask.handle.as_mut());
            Ok(mask)
        }
    }

    /// Allocate a new access mask. The state of it will be uninitialized.
    pub(super) unsafe fn allocate() -> Result<Self> {
        let mut handle = mem::MaybeUninit::uninit();
        errno!(alsa::snd_pcm_access_mask_malloc(handle.as_mut_ptr()))?;
        let handle = ptr::NonNull::new_unchecked(handle.assume_init());
        Ok(Self { handle })
    }

    /// Test if mask is empty.
    ///
    /// See [AccessMask] documentation.
    pub fn is_empty(&self) -> bool {
        unsafe { alsa::snd_pcm_access_mask_empty(self.handle.as_ptr()) == 1 }
    }

    /// Set all bits.
    ///
    /// See [AccessMask] documentation.
    pub fn any(&mut self) {
        unsafe {
            alsa::snd_pcm_access_mask_any(self.handle.as_mut());
        }
    }

    /// Reset all bits.
    ///
    /// See [AccessMask] documentation.
    pub fn none(&mut self) {
        unsafe {
            alsa::snd_pcm_access_mask_none(self.handle.as_mut());
        }
    }

    /// Make an access type present.
    ///
    /// See [AccessMask] documentation.
    pub fn set(&mut self, access: Access) {
        unsafe {
            alsa::snd_pcm_access_mask_set(self.handle.as_mut(), access as c::c_uint);
        }
    }

    /// Make an access type missing.
    ///
    /// See [AccessMask] documentation.
    pub fn reset(&mut self, access: Access) {
        unsafe {
            alsa::snd_pcm_access_mask_reset(self.handle.as_mut(), access as c::c_uint);
        }
    }

    /// Test the presence of an access type.
    ///
    /// See [AccessMask] documentation.
    pub fn test(&mut self, access: Access) -> bool {
        unsafe { alsa::snd_pcm_access_mask_test(self.handle.as_mut(), access as c::c_uint) == 1 }
    }

    /// Copy one mask to another.
    pub fn copy(&mut self, other: &Self) {
        unsafe {
            alsa::snd_pcm_access_mask_copy(self.handle.as_mut(), other.handle.as_ptr());
        }
    }
}

impl Drop for AccessMask {
    fn drop(&mut self) {
        unsafe {
            let _ = alsa::snd_pcm_access_mask_free(self.handle.as_mut());
        }
    }
}
