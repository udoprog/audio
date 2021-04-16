//! If any available, this provides handles for various forms of asynchronous
//! drivers that can be used in combination with audio interfaces.

mod atomic_waker;
use crate::Result;
use std::cell::Cell;
use std::future::Future;
use std::ptr;

thread_local! {
    static RUNTIME: Cell<*const Runtime> = Cell::new(ptr::null());
}

cfg_events_driver! {
    pub(crate) mod events;
    #[doc(hidden)]
    pub use self::events::EventsDriver;

    pub(crate) fn with_events<F, T>(f: F) -> T where F: FnOnce(&EventsDriver) -> T {
        RUNTIME.with(|rt| {
            // Safety: we maintain tight control of how and when RUNTIME is
            // constructed.
            let rt = unsafe { rt.get().as_ref().expect("missing audio runtime") };
            f(&rt.events)
        })
    }
}

cfg_poll_driver! {
    pub(crate) mod poll;
    #[doc(hidden)]
    pub use self::poll::{PollDriver, AsyncPoll};

    pub(crate) fn with_poll<F, T>(f: F) -> T where F: FnOnce(&PollDriver) -> T {
        RUNTIME.with(|rt| {
            // Safety: we maintain tight control of how and when RUNTIME is
            // constructed.
            let rt = unsafe { rt.get().as_ref().expect("missing audio runtime") };
            f(&rt.poll)
        })
    }
}

/// The audio runtime.
///
/// This is necessary to use with asynchronous audio-related APIs.
///
/// To run an asynchronous task inside of the audio runtime, we use the
/// [wrap][Runtime::wrap] function.
///
/// # Examples
///
/// ```rust,no_run
/// # async fn task() {}
/// # #[tokio::main] async fn main() -> anyhow::Result<()> {
/// let runtime = audio_device::runtime::Runtime::new()?;
/// runtime.wrap(task()).await;
/// # Ok(()) }
/// ```
pub struct Runtime {
    #[cfg(feature = "events-driver")]
    events: self::events::EventsDriver,
    #[cfg(feature = "poll-driver")]
    poll: self::poll::PollDriver,
}

impl Runtime {
    /// Construct a new audio runtime.
    pub fn new() -> Result<Self> {
        Ok(Self {
            #[cfg(feature = "events-driver")]
            events: self::events::EventsDriver::new()?,
            #[cfg(feature = "poll-driver")]
            poll: self::poll::PollDriver::new()?,
        })
    }

    /// Construct a runtime guard that when in scope will provide thread-local
    /// access to runtime drivers.
    pub fn enter(&self) -> RuntimeGuard<'_> {
        let old = RUNTIME.with(|rt| rt.replace(self as *const _));

        RuntimeGuard {
            _runtime: self,
            old,
        }
    }

    /// Run the given asynchronous task inside of the runtime.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # async fn task() {}
    /// # #[tokio::main] async fn main() -> anyhow::Result<()> {
    /// let runtime = audio_device::runtime::Runtime::new()?;
    /// runtime.wrap(task()).await;
    /// # Ok(()) }
    /// ```
    pub async fn wrap<F>(&self, f: F) -> F::Output
    where
        F: Future,
    {
        use std::pin::Pin;
        use std::task::{Context, Poll};

        return GuardFuture(self, f).await;

        struct GuardFuture<'a, F>(&'a Runtime, F);

        impl<'a, F> Future for GuardFuture<'a, F>
        where
            F: Future,
        {
            type Output = F::Output;

            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                let _guard = self.0.enter();
                let future = unsafe { Pin::map_unchecked_mut(self, |this| &mut this.1) };
                future.poll(cx)
            }
        }
    }

    /// Shutdown and join the runtime.
    pub fn join(self) {
        #[cfg(feature = "events-driver")]
        let _ = self.events.join();
        #[cfg(feature = "poll-driver")]
        let _ = self.poll.join();
    }
}

/// The runtime guard constructed with [Runtime::enter].
///
/// Runtime plumbing is available as long as this guard is in scope.
pub struct RuntimeGuard<'a> {
    // NB: prevent the guard from outliving the runtime it was constructed from.
    _runtime: &'a Runtime,
    old: *const Runtime,
}

impl Drop for RuntimeGuard<'_> {
    fn drop(&mut self) {
        RUNTIME.with(|rt| {
            rt.set(self.old);
        })
    }
}
