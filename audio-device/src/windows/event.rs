use crate::windows::RawEvent;
use std::ptr;
use windows_sys::Windows::Win32::SystemServices as ss;
use windows_sys::Windows::Win32::WindowsProgramming as wp;

const NULL: ss::HANDLE = ss::HANDLE(0);

/// A managed ewvent object.
#[repr(transparent)]
pub struct Event {
    handle: ss::HANDLE,
}

impl Event {
    pub(crate) fn new(manual_reset: bool, initial_state: bool) -> windows::Result<Self> {
        let handle = unsafe {
            ss::CreateEventA(
                ptr::null_mut(),
                manual_reset,
                initial_state,
                ss::PSTR::default(),
            )
        };

        if handle == NULL {
            return Err(windows::Error::new(
                windows::HRESULT::from_thread(),
                "failed to create event handle",
            ));
        }

        Ok(Self { handle })
    }

    /// Set the event.
    pub fn set(&self) {
        unsafe {
            ss::SetEvent(self.handle);
        }
    }

    /// Reset the event.
    pub fn reset(&self) {
        unsafe {
            ss::ResetEvent(self.handle);
        }
    }
}

impl RawEvent for Event {
    unsafe fn raw_event(&self) -> ss::HANDLE {
        self.handle
    }
}

impl Drop for Event {
    fn drop(&mut self) {
        unsafe {
            // NB: We intentionally ignore errors here.
            let _ = wp::CloseHandle(self.handle).ok();
        }
    }
}

unsafe impl Send for Event {}
unsafe impl Sync for Event {}
