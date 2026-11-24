use core::error::Error as CoreError;
use core::fmt;

#[cfg(feature = "wasapi")]
use windows::core::Error as WindowsError;

#[cfg(any(feature = "pulse", feature = "alsa"))]
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
            #[cfg(any(feature = "pulse", feature = "alsa"))]
            ErrorKind::Unix(ref error) => Some(error),
            #[cfg(feature = "wasapi")]
            ErrorKind::Windows(ref error) => Some(error),
        }
    }
}

#[derive(Debug)]
enum ErrorKind {
    #[cfg(any(feature = "pulse", feature = "alsa"))]
    /// A unix system error.
    Unix(Errno),
    #[cfg(feature = "wasapi")]
    /// A windows system error.
    Windows(WindowsError),
}

impl fmt::Display for ErrorKind {
    #[inline]
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            #[cfg(any(feature = "pulse", feature = "alsa"))]
            Self::Unix(ref error) => write!(_f, "unix system error: {error}"),
            #[cfg(feature = "wasapi")]
            Self::Windows(ref error) => write!(_f, "windows system error: {error}"),
        }
    }
}

#[cfg(any(feature = "pulse", feature = "alsa"))]
impl From<Errno> for Error {
    #[inline]
    fn from(errno: Errno) -> Self {
        Self {
            kind: ErrorKind::Unix(errno),
        }
    }
}

#[cfg(feature = "wasapi")]
impl From<WindowsError> for Error {
    #[inline]
    fn from(error: WindowsError) -> Self {
        Self {
            kind: ErrorKind::Windows(error),
        }
    }
}
