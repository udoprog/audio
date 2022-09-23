//! An idiomatic Rust ALSA interface.
// Documentation: https://www.alsa-project.org/alsa-doc/alsa-lib/

use crate::libc as c;
use crate::unix::Errno;
use std::io;
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

/// Errors that can be raised by the ALSA layer.
#[derive(Debug, Error)]
pub enum Error {
    /// System error.
    #[error("system error: {0}")]
    Sys(#[from] Errno),
    /// I/O error.
    #[error("i/o error: {0}")]
    Io(
        #[source]
        #[from]
        io::Error,
    ),
    /// Error raised when there's a format mismatch between an underlying stream
    /// and the type attempting to be used with it.
    #[error("type `{ty}` is not appropriate to use with format `{format}`")]
    FormatMismatch {
        /// A description of the type expected.
        ty: &'static str,
        /// The format that mismatched.
        format: Format,
    },
    /// Error raised when there's a channel count mismatch between an underlying
    /// stream and the type attempting to be used with it.
    #[error("mismatch in number of channels in buffer; actual = {actual}, expected = {expected}")]
    ChannelsMismatch {
        /// The actual number of channels.
        actual: usize,
        /// The expected number of channels.
        expected: usize,
    },
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
    /// Underlying PCM was not set up for polling.
    #[error("pcm device is not pollable")]
    MissingPollFds,
}

/// Helper result wrapper.
pub type Result<T, E = Error> = ::std::result::Result<T, E>;

mod card;
pub use self::card::{cards, Card};

mod pcm;
pub use self::pcm::Pcm;

mod hardware_parameters;
pub use self::hardware_parameters::{HardwareParameters, HardwareParametersMut};

mod software_parameters;
pub use self::software_parameters::{SoftwareParameters, SoftwareParametersMut};

mod format_mask;
pub use self::format_mask::FormatMask;

mod access_mask;
pub use self::access_mask::AccessMask;

mod enums;
pub use self::enums::{
    Access, ControlElementInterface, Direction, Format, State, Stream, Timestamp, TimestampType,
};

mod channel_area;
#[doc(hidden)]
pub use self::channel_area::ChannelArea;

mod writer;
pub use self::writer::Writer;

cfg_poll_driver! {
    mod async_writer;
    pub use self::async_writer::AsyncWriter;
}

mod sample;
pub use self::sample::Sample;

mod configurator;
pub use self::configurator::{Config, Configurator};

mod control;
pub use self::control::{Control, ControlElementList};
