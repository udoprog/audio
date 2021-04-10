#[cfg(loom)]
pub(crate) use loom::sync;
#[cfg(loom)]
pub(crate) use loom::thread;

#[cfg(not(loom))]
pub(crate) use ::std::sync;
#[cfg(not(loom))]
pub(crate) use ::std::thread;
