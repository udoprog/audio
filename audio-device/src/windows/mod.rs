//! Shared helpers for windows programming.

use windows::Win32::Foundation::HANDLE;

mod event;
pub use self::event::Event;

#[cfg(feature = "events-driver")]
#[cfg_attr(docsrs, doc(cfg(feature = "events-driver")))]
pub use crate::runtime::events::AsyncEvent;

/// Trait that indicates a type that encapsulates an event.
pub trait RawEvent {
    /// Access the underlying raw handle for the event.
    ///
    /// # Safety
    ///
    /// Caller must ensure that the raw handle stays alive for the duration of
    /// whatever its being associated with.
    unsafe fn raw_event(&self) -> HANDLE;
}
