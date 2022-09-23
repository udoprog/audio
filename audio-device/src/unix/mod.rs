//! Unix-specific types and definitions.

use std::{fmt, error};

/// A unix error number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Errno(i32);

impl Errno {
    pub(crate) const EWOULDBLOCK: Self = Self(libc::EWOULDBLOCK);

    pub(crate) fn new(value: i32) -> Self {
        Self(value)
    }
}

impl fmt::Display for Errno {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::EWOULDBLOCK => {
                write!(f, "EWOULDBLOCK")
            }
            errno => {
                write!(f, "({})", errno)
            }
        }
    }
}

impl error::Error for Errno {
}

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
