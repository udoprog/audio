use windows::Win32::System::Threading as th;
use windows::Win32::Foundation as f;
use windows::core::PCSTR;

use crate::windows::RawEvent;

/// A managed ewvent object.
#[repr(transparent)]
pub struct Event {
    handle: f::HANDLE,
}

impl Event {
    pub(crate) fn new(manual_reset: bool, initial_state: bool) -> windows::core::Result<Self> {
        let handle = unsafe {
            th::CreateEventA(
                None,
                manual_reset,
                initial_state,
                PCSTR::null(),
            )?
        };

        Ok(Self { handle })
    }

    /// Set the event.
    pub fn set(&self) {
        unsafe {
            th::SetEvent(self.handle);
        }
    }

    /// Reset the event.
    pub fn reset(&self) {
        unsafe {
            th::ResetEvent(self.handle);
        }
    }
}

impl RawEvent for Event {
    unsafe fn raw_event(&self) -> f::HANDLE {
        self.handle
    }
}

impl Drop for Event {
    fn drop(&mut self) {
        unsafe {
            // NB: We intentionally ignore errors here.
            let _ = f::CloseHandle(self.handle).ok();
        }
    }
}

unsafe impl Send for Event {}
unsafe impl Sync for Event {}
