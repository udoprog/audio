use core::ffi::{CStr, c_char};
use core::ops;

/// A string allocated through libc.
#[repr(transparent)]
pub struct CString {
    ptr: *mut c_char,
}

// Safety: string is allocated with the libc allocator and can be freely shared
// across threads.
unsafe impl Send for CString {}
unsafe impl Sync for CString {}

impl CString {
    /// Construct a new string that was allocated through libc.
    ///
    /// This differs from [std::ffi::CString] in that it requires the underlying
    /// string to have been allocated using libc allocators, and will free the
    /// underlying string using those as well.
    pub unsafe fn from_raw(ptr: *mut c_char) -> Self {
        Self { ptr }
    }
}

impl Drop for CString {
    fn drop(&mut self) {
        unsafe {
            libc::free(self.ptr.cast());
        }
    }
}

impl ops::Deref for CString {
    type Target = CStr;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { CStr::from_ptr(self.ptr) }
    }
}
