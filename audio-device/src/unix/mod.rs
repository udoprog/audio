//! Unix-specific types and definitions.

use core::error::Error as CoreError;
use core::fmt;

/// A unix error number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Errno(i32);

impl Errno {
    #[cfg(feature = "alsa")]
    pub(crate) const EWOULDBLOCK: Self = Self(libc::EWOULDBLOCK);

    pub(crate) fn new(value: i32) -> Self {
        Self(value)
    }
}

impl fmt::Display for Errno {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            libc::EPERM => write!(f, "EPERM"),
            libc::ENOENT => write!(f, "ENOENT"),
            libc::ESRCH => write!(f, "ESRCH"),
            libc::EINTR => write!(f, "EINTR"),
            libc::EIO => write!(f, "EIO"),
            libc::ENXIO => write!(f, "ENXIO"),
            libc::E2BIG => write!(f, "E2BIG"),
            libc::ENOEXEC => write!(f, "ENOEXEC"),
            libc::EBADF => write!(f, "EBADF"),
            libc::ECHILD => write!(f, "ECHILD"),
            libc::EAGAIN => write!(f, "EAGAIN"),
            libc::ENOMEM => write!(f, "ENOMEM"),
            libc::EACCES => write!(f, "EACCES"),
            libc::EFAULT => write!(f, "EFAULT"),
            libc::ENOTBLK => write!(f, "ENOTBLK"),
            libc::EBUSY => write!(f, "EBUSY"),
            libc::EEXIST => write!(f, "EEXIST"),
            libc::EXDEV => write!(f, "EXDEV"),
            libc::ENODEV => write!(f, "ENODEV"),
            libc::ENOTDIR => write!(f, "ENOTDIR"),
            libc::EISDIR => write!(f, "EISDIR"),
            libc::EINVAL => write!(f, "EINVAL"),
            libc::ENFILE => write!(f, "ENFILE"),
            libc::EMFILE => write!(f, "EMFILE"),
            libc::ENOTTY => write!(f, "ENOTTY"),
            libc::ETXTBSY => write!(f, "ETXTBSY"),
            libc::EFBIG => write!(f, "EFBIG"),
            libc::ENOSPC => write!(f, "ENOSPC"),
            libc::ESPIPE => write!(f, "ESPIPE"),
            libc::EROFS => write!(f, "EROFS"),
            libc::EMLINK => write!(f, "EMLINK"),
            libc::EPIPE => write!(f, "EPIPE"),
            libc::EDOM => write!(f, "EDOM"),
            libc::ERANGE => write!(f, "ERANGE"),
            errno => {
                write!(f, "UNKNOWN({})", errno)
            }
        }
    }
}

impl CoreError for Errno {}

cfg_poll_driver! {
    /// Poll flags.
    #[derive(Debug, Clone, Copy)]
    #[repr(transparent)]
    pub struct PollFlags(libc::c_short);

    impl PollFlags {
        pub(crate) const POLLOUT: Self = Self(crate::libc::POLLOUT);

        pub(crate) fn from_bits_truncate(bits: libc::c_short) -> Self {
            Self(bits)
        }

        pub(crate) fn test(self, bits: PollFlags) -> bool {
            (self.0 & bits.0) != 0
        }
    }

    pub use crate::runtime::poll::{AsyncPoll, PollEventsGuard};
}
