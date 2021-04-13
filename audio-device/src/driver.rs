//! If any available, this provides handles for various forms of asynchronous
//! drivers that can be used in combination with audio interfaces.

mod atomic_waker;
#[cfg(windows)]
pub mod events;
