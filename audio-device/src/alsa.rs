//! An idiomatic Rust ALSA interface.

pub use nix::errno::Errno;
use std::ops;
use thiserror::Error;

/// A string allocated through libc.
#[repr(transparent)]
pub struct CString {
    ptr: *mut libc::c_char,
}

impl CString {
    /// Construct a new string that was allocated through libc.
    ///
    /// This differs from [std::ffi::CString] in that it requires the underlying
    /// string to have been allocated using libc allocators, and will free the
    /// underlying string using those as well.
    pub unsafe fn from_raw(ptr: *mut libc::c_char) -> Self {
        Self { ptr }
    }
}

impl Drop for CString {
    fn drop(&mut self) {
        unsafe {
            libc::free(self.ptr as *mut _);
        }
    }
}

impl ops::Deref for CString {
    type Target = std::ffi::CStr;

    fn deref(&self) -> &Self::Target {
        unsafe { std::ffi::CStr::from_ptr(self.ptr) }
    }
}

// Safety: string is allocated with the libc allocator and can be freely shared
// across threads.
unsafe impl Send for CString {}
unsafe impl Sync for CString {}

macro_rules! errno {
    ($expr:expr) => {{
        let result: i32 = $expr;

        if result < 0 {
            Err($crate::alsa::Error::Errno(::nix::errno::Errno::from_i32(
                -result,
            )))
        } else {
            Ok(result)
        }
    }};
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("system error: {0}")]
    Errno(Errno),
    #[error("bad format identifier ({0})")]
    BadFormat(i32),
    #[error("bad access identifier ({0})")]
    BadAccess(u32),
}

/// Helper result wrapper.
pub type Result<T, E = Error> = ::std::result::Result<T, E>;

mod card;
pub use self::card::{cards, Card};

mod pcm;
pub use self::pcm::{Pcm, Stream};

mod hardware_parameters;
pub use self::hardware_parameters::{Direction, HardwareParametersAny, HardwareParametersCurrent};

mod format_mask;
pub use self::format_mask::FormatMask;

mod access_mask;
pub use self::access_mask::AccessMask;

mod enums;
pub use self::enums::{Access, Format};
