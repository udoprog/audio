use core::cell::Cell;
use core::error::Error as CoreError;
use core::ffi::c_uint;
use core::fmt;
use core::ptr;

use alloc::boxed::Box;

use std::thread_local;

use crate::unix::Errno;

macro_rules! __error {
    ($s:expr, $expr:expr) => {{
        let result = $expr;

        if result < 0 {
            let errno = { pulse::pa_context_errno($s.handle.as_ptr()) };
            Err(crate::pulse::Error::from(crate::unix::Errno::new(errno)))
        } else {
            $crate::pulse::error::ffi_error!(result)
        }
    }};
}

macro_rules! __ffi_error {
    ($expr:expr) => {{
        let result = $expr;

        if let Some(e) = $crate::pulse::error::last_error() {
            Err(e)
        } else {
            Ok(result)
        }
    }};
}

pub(crate) use __error as error;
pub(crate) use __ffi_error as ffi_error;

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

/// PulseAudio-specific result alias.
pub type Result<T, E = Error> = ::core::result::Result<T, E>;

/// PulseAudio-specific errors.
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    /// Create a new user-defined error.
    pub fn user<E>(error: E) -> Self
    where
        E: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        Self {
            kind: ErrorKind::User(Box::new(DisplayError(error))),
        }
    }
}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}

impl fmt::Debug for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}

impl CoreError for Error {
    #[inline]
    fn source(&self) -> Option<&(dyn CoreError + 'static)> {
        match self.kind {
            ErrorKind::Errno(ref error) => Some(error),
            ErrorKind::BadContextState(..) => None,
            ErrorKind::User(ref error) => Some(&**error),
        }
    }
}

impl From<Errno> for Error {
    #[inline]
    fn from(errno: Errno) -> Self {
        Self {
            kind: ErrorKind::Errno(errno),
        }
    }
}

impl From<ErrorKind> for Error {
    #[inline]
    fn from(kind: ErrorKind) -> Self {
        Self { kind }
    }
}

#[derive(Debug)]
pub(crate) enum ErrorKind {
    /// System error.
    Errno(Errno),
    /// Tried to decode bad context state.
    BadContextState(c_uint),
    /// A custom user error.
    User(Box<dyn CoreError + Send + Sync + 'static>),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Errno(errno) => {
                write!(f, "system error: {errno}")
            }
            Self::BadContextState(state) => {
                write!(f, "bad context state identifier `{state}`")
            }
            Self::User(ref error) => error.fmt(f),
        }
    }
}

#[repr(transparent)]
struct DisplayError<E>(E);

impl<E> fmt::Display for DisplayError<E>
where
    E: fmt::Display,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<E> fmt::Debug for DisplayError<E>
where
    E: fmt::Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<E> CoreError for DisplayError<E> where E: fmt::Display + fmt::Debug {}
