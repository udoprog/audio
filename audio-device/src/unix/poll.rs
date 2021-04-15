//! Unix-related types needed to deal with polling.

use crate::libc as c;
use crate::unix::errno;
use std::os::unix::io::{AsRawFd, RawFd};

#[doc(inline)]
pub use ::nix::poll::PollFlags;

/// `poll` waits for one of a set of file descriptors to become ready to perform
/// I/O.
/// ([`poll(2)`](http://pubs.opengroup.org/onlinepubs/9699919799/functions/poll.html))
///
/// `fds` contains all [`PollFd`](struct.PollFd.html) to poll. The function will
/// return as soon as any event occur for any of these `PollFd`s.
///
/// The `timeout` argument specifies the number of milliseconds that `poll()`
/// should block waiting for a file descriptor to become ready.  The call will
/// block until either:
///
/// *  a file descriptor becomes ready;
/// *  the call is interrupted by a signal handler; or
/// *  the timeout expires.
///
/// Note that the timeout interval will be rounded up to the system clock
/// granularity, and kernel scheduling delays mean that the blocking interval
/// may overrun by a small amount.  Specifying a negative value in timeout means
/// an infinite timeout.  Specifying a timeout of zero causes `poll()` to return
/// immediately, even if no file descriptors are ready.
pub fn poll(fds: &mut [PollFd], timeout: c::c_int) -> Result<c::c_int, errno::Errno> {
    let res = unsafe {
        c::poll(
            fds.as_mut_ptr() as *mut c::pollfd,
            fds.len() as c::nfds_t,
            timeout,
        )
    };

    if res < 0 {
        return Err(errno::Errno::from_i32(-res));
    }

    Ok(res)
}

/// This is a wrapper around `libc::pollfd`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PollFd {
    pollfd: c::pollfd,
}

impl PollFd {
    /// Creates a new `PollFd` specifying the events of interest
    /// for a given file descriptor.
    pub fn new(fd: RawFd, events: PollFlags) -> PollFd {
        PollFd {
            pollfd: c::pollfd {
                fd,
                events: events.bits(),
                revents: PollFlags::empty().bits(),
            },
        }
    }

    /// Returns the events that occured in the last call to `poll` or `ppoll`.
    pub fn revents(self) -> Option<PollFlags> {
        PollFlags::from_bits(self.pollfd.revents)
    }
}

impl AsRawFd for PollFd {
    fn as_raw_fd(&self) -> RawFd {
        self.pollfd.fd
    }
}
