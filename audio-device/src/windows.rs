//! Shared helpers for windows programming.

use windows_sys::Windows::Win32::SystemServices as ss;
mod event;
pub use self::event::{AsyncEvent, Event};

/// Trait that indicates a type that encapsulates an event.
pub trait RawEvent {
    /// Access the underlying raw handle for the event.
    ///
    /// # Safety
    ///
    /// Caller must ensure that the raw handle stays alive for the duration of
    /// whatever its being associated with.
    unsafe fn raw_event(&self) -> ss::HANDLE;
}
