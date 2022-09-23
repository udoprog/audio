//! libc specifics
//!
//! These are all re-exports from the [libc crate] and are intended for local
//! use w/ APIs that uses a C-like ABI, like [ALSA][crate::alsa].
//!
//! [libc crate]: https://crates.io/crates/libc

pub use ::libc::eventfd;
pub use ::libc::free;
pub use ::libc::nfds_t;
pub use ::libc::{EFD_NONBLOCK, EWOULDBLOCK};
pub use ::libc::{c_char, c_int, c_long, c_short, c_uint, c_ulong, c_void};
pub use ::libc::{poll, pollfd, POLLIN, POLLOUT};
pub use ::libc::{read, write};
