use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Threading::{CreateEventA, ResetEvent, SetEvent};
use windows::core::{PCSTR, Result};

use crate::windows::RawEvent;

/// A managed ewvent object.
#[repr(transparent)]
pub struct Event {
    handle: HANDLE,
}

impl Event {
    pub(crate) fn new(manual_reset: bool, initial_state: bool) -> Result<Self> {
        let handle = unsafe { CreateEventA(None, manual_reset, initial_state, PCSTR::null())? };
        Ok(Self { handle })
    }

    /// Set the event.
    pub fn set(&self) -> Result<()> {
        unsafe { SetEvent(self.handle) }
    }

    /// Reset the event.
    pub fn reset(&self) -> Result<()> {
        unsafe { ResetEvent(self.handle) }
    }
}

impl RawEvent for Event {
    unsafe fn raw_event(&self) -> HANDLE {
        self.handle
    }
}

impl Drop for Event {
    fn drop(&mut self) {
        unsafe {
            // NB: We intentionally ignore errors here.
            let _ = CloseHandle(self.handle);
        }
    }
}

unsafe impl Send for Event {}
unsafe impl Sync for Event {}
