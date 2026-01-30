use core::error::Error as CoreError;
use core::fmt;

#[cfg(feature = "windows")]
use windows::core::Error as WindowsError;

#[cfg(feature = "unix")]
use crate::unix::Errno;

/// Audio device-specific result alias.
pub type Result<T, E = Error> = ::core::result::Result<T, E>;

/// Audio device-specific errors.
pub struct Error {
    kind: ErrorKind,
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
            #[cfg(feature = "unix")]
            ErrorKind::Unix(ref error) => Some(error),
            #[cfg(feature = "windows")]
            ErrorKind::Windows(ref error) => Some(error),
        }
    }
}

#[derive(Debug)]
enum ErrorKind {
    #[cfg(feature = "unix")]
    /// A unix system error.
    Unix(Errno),
    #[cfg(feature = "windows")]
    /// A windows system error.
    Windows(WindowsError),
}

impl fmt::Display for ErrorKind {
    #[inline]
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            #[cfg(feature = "unix")]
            Self::Unix(ref error) => write!(_f, "unix system error: {error}"),
            #[cfg(feature = "windows")]
            Self::Windows(ref error) => write!(_f, "windows system error: {error}"),
        }
    }
}

#[cfg(feature = "unix")]
impl From<Errno> for Error {
    #[inline]
    fn from(errno: Errno) -> Self {
        Self {
            kind: ErrorKind::Unix(errno),
        }
    }
}

#[cfg(feature = "windows")]
impl From<WindowsError> for Error {
    #[inline]
    fn from(error: WindowsError) -> Self {
        Self {
            kind: ErrorKind::Windows(error),
        }
    }
}
