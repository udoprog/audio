use crate::libc as c;
use crate::unix::Errno;
use std::cell::Cell;
use std::ptr;
use thiserror::Error;

macro_rules! error {
    ($s:expr, $expr:expr) => {{
        let result = $expr;

        if result < 0 {
            let errno = { pulse::pa_context_errno($s.handle.as_ptr()) };

            Err(crate::pulse::Error::Sys(
                crate::unix::Errno::new(errno),
            ))
        } else {
            ffi_error!(result)
        }
    }};
}

macro_rules! ffi_error {
    ($expr:expr) => {{
        let result = $expr;

        if let Some(e) = $crate::pulse::error::last_error() {
            Err(e)
        } else {
            Ok(result)
        }
    }};
}

thread_local! {
    /// The last error encountered on this thread.
    ///
    /// This is set by callbacks so transfer errors across their FFI boundary.
    static LAST_ERROR: Cell<*mut Error> = Cell::new(ptr::null_mut());
}

/// Take the last error encountered on this thread.
pub(super) fn last_error() -> Option<Error> {
    LAST_ERROR.with(|e| {
        let e = e.replace(ptr::null_mut());

        if e.is_null() {
            None
        } else {
            // Safety: fully managed within this module.
            Some(unsafe { *Box::from_raw(e) })
        }
    })
}

/// Run the given closure and capture any errors raised.
///
/// Also abort on panics.
pub(super) fn capture<F>(f: F)
where
    F: FnOnce() -> Result<()>,
{
    if let Err(e) = f() {
        let new = Box::into_raw(Box::new(e));

        LAST_ERROR.with(|e| {
            let e = e.replace(new);

            if !e.is_null() {
                // Safety: fully managed within this module.
                let _ = unsafe { Box::from_raw(e) };
            }
        });
    }
}

/// Errors that can be raised by the PulseAudio layer.
#[derive(Debug, Error)]
pub enum Error {
    /// System error.
    #[error("system error: {0}")]
    Sys(#[from] Errno),
    /// Tried to decode bad context state.
    #[error("bad context state identifier `{0}`")]
    BadContextState(c::c_uint),
    /// A custom user error.
    #[error("error: {0}")]
    User(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
}

/// Helper result wrapper.
pub type Result<T, E = Error> = ::std::result::Result<T, E>;
