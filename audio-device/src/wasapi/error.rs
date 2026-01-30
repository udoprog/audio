use core::error::Error as CoreError;
use core::fmt;

use windows::core::Error as WindowsError;

//// WASAPI-specific result alias.
pub type Result<T, E = Error> = ::core::result::Result<T, E>;

/// WASAPI-specific errors.
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
            ErrorKind::CreateInstance(ref error) => Some(error),
            ErrorKind::WaitError(ref error) => Some(error),
            ErrorKind::IsFormatSupported(ref error) => Some(error),
            ErrorKind::GetBufferSize(ref error) => Some(error),
            ErrorKind::GetBuffer(ref error) => Some(error),
            ErrorKind::ReleaseBuffer(ref error) => Some(error),
            ErrorKind::GetCurrentPadding(ref error) => Some(error),
            ErrorKind::Start(ref error) => Some(error),
            ErrorKind::Stop(ref error) => Some(error),
            ErrorKind::Initialize(ref error) => Some(error),
            ErrorKind::MakeEvent(ref error) => Some(error),
            ErrorKind::GetService(ref error) => Some(error),
            ErrorKind::GetMixFormat(ref error) => Some(error),
            ErrorKind::SetEventHandle(ref error) => Some(error),
            ErrorKind::UnsupportedMixFormat => None,
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
    CreateInstance(WindowsError),
    WaitError(WindowsError),
    IsFormatSupported(WindowsError),
    GetBufferSize(WindowsError),
    GetBuffer(WindowsError),
    ReleaseBuffer(WindowsError),
    GetCurrentPadding(WindowsError),
    Start(WindowsError),
    Stop(WindowsError),
    Initialize(WindowsError),
    MakeEvent(WindowsError),
    GetService(WindowsError),
    GetMixFormat(WindowsError),
    SetEventHandle(WindowsError),
    UnsupportedMixFormat,
}

impl fmt::Display for ErrorKind {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CreateInstance(error) => {
                write!(f, "Error creating COM instance: {error}")
            }
            Self::WaitError(error) => {
                write!(f, "Error waiting for event: {error}")
            }
            Self::IsFormatSupported(error) => {
                write!(f, "Error checking format support: {error}")
            }
            Self::GetBufferSize(error) => {
                write!(f, "Error getting buffer size: {error}")
            }
            Self::GetBuffer(error) => {
                write!(f, "Error getting buffer from render client: {error}")
            }
            Self::ReleaseBuffer(error) => {
                write!(f, "Error releasing buffer to render client: {error}")
            }
            Self::GetCurrentPadding(error) => {
                write!(
                    f,
                    "Error getting current padding from render client: {error}"
                )
            }
            Self::Start(error) => {
                write!(f, "Error starting audio client: {error}")
            }
            Self::Stop(error) => {
                write!(f, "Error stopping audio client: {error}")
            }
            Self::Initialize(error) => {
                write!(f, "Error initializing audio client: {error}")
            }
            Self::MakeEvent(error) => {
                write!(f, "Error creating event: {error}")
            }
            Self::GetService(error) => {
                write!(f, "Error getting audio service: {error}")
            }
            Self::GetMixFormat(error) => {
                write!(f, "Error getting device mix format: {error}")
            }
            Self::SetEventHandle(error) => {
                write!(f, "Error setting event handle: {error}")
            }
            Self::UnsupportedMixFormat => {
                write!(f, "Device doesn't support a compatible mix format")
            }
        }
    }
}
