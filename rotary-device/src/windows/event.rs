use crate::bindings::Windows::Win32::SystemServices as ss;
use crate::bindings::Windows::Win32::WindowsProgramming as wp;
use std::ptr;

const NULL: ss::HANDLE = unsafe { std::mem::transmute(0usize) };

/// A reference counted event object.
#[repr(transparent)]
pub(crate) struct Event {
    handle: ss::HANDLE,
}

impl Event {
    pub(crate) fn create_event(manual_reset: bool, initial_state: bool) -> windows::Result<Self> {
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
                windows::ErrorCode::from_thread(),
                "failed to create event handle",
            ));
        }

        Ok(Self { handle })
    }

    /// Return the raw pointer to the underlying handle.
    ///
    /// # Safety
    ///
    /// Caller must ensure that this event handle stays alive for the duration
    /// of whatever its being associated with.
    pub(crate) unsafe fn handle(&self) -> ss::HANDLE {
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
