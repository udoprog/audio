//! libc specifics
//!
//! These are all re-exports from the [libc crate] and are intended for local
//! use w/ APIs that uses a C-like ABI, like [ALSA][crate::alsa].
//!
//! [libc crate]: https://crates.io/crates/libc

pub use ::libc::eventfd;
pub use ::libc::nfds_t;
pub use ::libc::{EFD_NONBLOCK, EWOULDBLOCK};
pub use ::libc::{POLLIN, POLLOUT, poll, pollfd};
pub use ::libc::{close, free};
pub use ::libc::{read, write};
