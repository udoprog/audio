use core::error::Error as CoreError;
use core::ffi::{c_int, c_uint};
use core::fmt;

use crate::alsa::Format;
use crate::unix::Errno;

/// ALSA-specific result alias.
pub type Result<T, E = Error> = ::core::result::Result<T, E>;

macro_rules! __errno {
    ($expr:expr) => {{
        let result = $expr;

        if result < 0 {
            Err($crate::unix::Errno::new(-result as i32))
        } else {
            Ok(result)
        }
    }};
}

pub(crate) use __errno as errno;

/// ALSA-specific errors.
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    #[cfg(feature = "alsa")]
    pub(crate) fn would_block(&self) -> bool {
        matches!(self.kind, ErrorKind::Errno(errno) if errno == Errno::EWOULDBLOCK)
    }
}

impl From<ErrorKind> for Error {
    #[inline]
    fn from(kind: ErrorKind) -> Self {
        Self { kind }
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
            ErrorKind::Errno(ref error) => Some(error),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub(crate) enum ErrorKind {
    /// System error.
    Errno(Errno),
    /// Error raised when there's a format mismatch between an underlying stream
    /// and the type attempting to be used with it.
    FormatMismatch {
        /// A description of the type expected.
        ty: &'static str,
        /// The format that mismatched.
        format: Format,
    },
    /// Error raised when there's a channel count mismatch between an underlying
    /// stream and the type attempting to be used with it.
    ChannelsMismatch {
        /// The actual number of channels.
        actual: usize,
        /// The expected number of channels.
        expected: usize,
    },
    /// Underlying function call returned an illegal format identifier.
    BadFormat(c_int),
    /// Underlying function call returned an illegal access identifier.
    BadAccess(c_uint),
    /// Underlying function call returned an illegal timestamp identifier.
    BadTimestamp(c_uint),
    /// Underlying function call returned an illegal timestamp type identifier.
    BadTimestampType(c_uint),
    /// Underlying PCM was not set up for polling.
    MissingPollFds,
}

impl fmt::Display for ErrorKind {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Errno(errno) => {
                write!(f, "system error: {errno}")
            }
            Self::FormatMismatch { ty, format } => {
                write!(
                    f,
                    "type `{ty}` is not appropriate to use with format `{format}`"
                )
            }
            Self::ChannelsMismatch { actual, expected } => {
                write!(
                    f,
                    "mismatch in number of channels in buffer; actual = {actual}, expected = {expected}"
                )
            }
            Self::BadFormat(format) => {
                write!(f, "bad format identifier ({format})")
            }
            Self::BadAccess(access) => {
                write!(f, "bad access identifier ({access})")
            }
            Self::BadTimestamp(timestamp) => {
                write!(f, "bad timestamp mode identifier ({timestamp})")
            }
            Self::BadTimestampType(ty) => {
                write!(f, "bad timestamp type identifier ({ty})")
            }
            Self::MissingPollFds => {
                write!(f, "pcm device is not pollable")
            }
        }
    }
}
