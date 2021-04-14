//! An idiomatic Rust ALSA interface.

use crate::libc as c;
use crate::unix::errno::Errno;
use std::ops;
use thiserror::Error;

/// A string allocated through libc.
#[repr(transparent)]
pub struct CString {
    ptr: *mut c::c_char,
}

impl CString {
    /// Construct a new string that was allocated through libc.
    ///
    /// This differs from [std::ffi::CString] in that it requires the underlying
    /// string to have been allocated using libc allocators, and will free the
    /// underlying string using those as well.
    pub unsafe fn from_raw(ptr: *mut c::c_char) -> Self {
        Self { ptr }
    }
}

impl Drop for CString {
    fn drop(&mut self) {
        unsafe {
            c::free(self.ptr as *mut _);
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
        let result = $expr;

        if result < 0 {
            Err($crate::alsa::Error::Sys(::nix::errno::Errno::from_i32(
                -result as i32,
            )))
        } else {
            Ok(result)
        }
    }};
}

#[derive(Debug, Error)]
pub enum Error {
    /// System error.
    #[error("system error: {0}")]
    Sys(Errno),
    #[error("type `{ty}` is not appropriate to use with format `{format}`")]
    FormatMismatch { ty: &'static str, format: Format },
    #[error("mismatch in number of channels in buffer; actual = {actual}, expected = {expected}")]
    ChannelsMismatch { actual: usize, expected: usize },
    /// Underlying function call returned an illegal format identifier.
    #[error("bad format identifier ({0})")]
    BadFormat(c::c_int),
    /// Underlying function call returned an illegal access identifier.
    #[error("bad access identifier ({0})")]
    BadAccess(c::c_uint),
    /// Underlying function call returned an illegal timestamp identifier.
    #[error("bad timestamp mode identifier ({0})")]
    BadTimestamp(c::c_uint),
    /// Underlying function call returned an illegal timestamp type identifier.
    #[error("bad timestamp type identifier ({0})")]
    BadTimestampType(c::c_uint),
}

/// Helper result wrapper.
pub type Result<T, E = Error> = ::std::result::Result<T, E>;

mod card;
pub use self::card::{cards, Card};

mod pcm;
pub use self::pcm::Pcm;

mod hardware_parameters;
pub use self::hardware_parameters::{Direction, HardwareParameters, HardwareParametersMut};

mod software_parameters;
pub use self::software_parameters::{SoftwareParameters, SoftwareParametersMut};

mod format_mask;
pub use self::format_mask::FormatMask;

mod access_mask;
pub use self::access_mask::AccessMask;

mod enums;
pub use self::enums::{Access, Format, Stream, Timestamp, TimestampType};

mod channel_area;
pub use self::channel_area::ChannelArea;

mod writer;
pub use self::writer::Writer;

mod sample;
pub use self::sample::Sample;

mod configurator;
pub use self::configurator::{Config, Configurator};
