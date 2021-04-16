use thiserror::Error;

/// Audio runtime errors.
#[derive(Debug, Error)]
pub enum Error {
    #[cfg(feature = "unix")]
    #[error("system error: {0}")]
    /// A unix system error.
    Unix(#[from] crate::unix::errno::Errno),
    #[cfg(feature = "windows")]
    #[error("system error: {0}")]
    /// A windows system error.
    Windows(
        #[from]
        #[source]
        windows::Error,
    ),
}

/// The re-exported error type.
pub type Result<T, E = Error> = ::std::result::Result<T, E>;
