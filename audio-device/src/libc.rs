//! libc specifics
//!
//! These are all re-exports from the [libc crate] and are intended for local
//! use w/ APIs that uses a C-like ABI, like [ALSA][crate::alsa].
//!
//! [libc crate]: https://crates.io/crates/libc

#[doc(inherit)]
pub use ::libc::eventfd;
#[doc(inherit)]
pub use ::libc::free;
#[doc(inherit)]
pub use ::libc::nfds_t;
#[doc(inherit)]
pub use ::libc::EFD_NONBLOCK;
#[doc(inherit)]
pub use ::libc::{c_char, c_int, c_long, c_short, c_uint, c_ulong, c_void};
#[doc(inherit)]
pub use ::libc::{poll, pollfd, POLLIN, POLLOUT};
#[doc(inherit)]
pub use ::libc::{read, write};
