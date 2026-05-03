use core::error::Error as CoreError;
use core::fmt;

use windows::core::Error as WindowsError;

/// WASAPI-specific result alias.
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
            ErrorKind::Activate(ref error) => Some(error),
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
    Activate(WindowsError),
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
            Self::CreateInstance { .. } => {
                write!(f, "Error creating COM instance")
            }
            Self::Activate { .. } => {
                write!(f, "Error activating COM instance")
            }
            Self::WaitError { .. } => {
                write!(f, "Error waiting for event")
            }
            Self::IsFormatSupported { .. } => {
                write!(f, "Error checking format support")
            }
            Self::GetBufferSize { .. } => {
                write!(f, "Error getting buffer size")
            }
            Self::GetBuffer { .. } => {
                write!(f, "Error getting buffer from render client")
            }
            Self::ReleaseBuffer { .. } => {
                write!(f, "Error releasing buffer to render client")
            }
            Self::GetCurrentPadding { .. } => {
                write!(f, "Error getting current padding from render client")
            }
            Self::Start { .. } => {
                write!(f, "Error starting audio client")
            }
            Self::Stop { .. } => {
                write!(f, "Error stopping audio client")
            }
            Self::Initialize { .. } => {
                write!(f, "Error initializing audio client")
            }
            Self::MakeEvent { .. } => {
                write!(f, "Error creating event")
            }
            Self::GetService { .. } => {
                write!(f, "Error getting audio service")
            }
            Self::GetMixFormat { .. } => {
                write!(f, "Error getting device mix format")
            }
            Self::SetEventHandle { .. } => {
                write!(f, "Error setting event handle")
            }
            Self::UnsupportedMixFormat => {
                write!(f, "Device doesn't support a compatible mix format")
            }
        }
    }
}
