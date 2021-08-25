//! Shared helpers for windows programming.

use windows_sys::Windows::Win32::Foundation as f;
mod event;
pub use self::event::Event;

cfg_events_driver! {
    pub use crate::runtime::events::AsyncEvent;
}

/// Trait that indicates a type that encapsulates an event.
pub trait RawEvent {
    /// Access the underlying raw handle for the event.
    ///
    /// # Safety
    ///
    /// Caller must ensure that the raw handle stays alive for the duration of
    /// whatever its being associated with.
    unsafe fn raw_event(&self) -> f::HANDLE;
}
