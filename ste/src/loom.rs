#[cfg(loom)]
pub use loom::sync;
#[cfg(loom)]
pub use loom::thread;

#[cfg(not(loom))]
pub use ::std::sync;
#[cfg(not(loom))]
pub use ::std::thread;
