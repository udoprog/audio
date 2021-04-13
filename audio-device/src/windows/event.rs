use crate::driver::events::{Shared, Waker};
use crate::loom::sync::atomic::Ordering;
use crate::windows::RawEvent;
use std::future::Future;
use std::pin::Pin;
use std::ptr;
use std::sync::Arc;
use std::task::{Context, Poll};
use windows_sys::Windows::Win32::SystemServices as ss;
use windows_sys::Windows::Win32::WindowsProgramming as wp;

const NULL: ss::HANDLE = ss::HANDLE(0);

/// A reference counted event object.
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
                windows::ErrorCode::from_thread(),
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

/// An asynchronous variant of [Event].
///
/// Constructed through [Handle::event][crate::driver::events::Handle::event]
pub struct AsyncEvent {
    shared: Arc<Shared>,
    waker: Arc<Waker>,
    event: Option<Event>,
}

impl AsyncEvent {
    /// Construct a new async event.
    pub(crate) fn new(shared: Arc<Shared>, waker: Arc<Waker>, event: Event) -> Self {
        Self {
            shared,
            waker,
            event: Some(event),
        }
    }

    /// Wait for the specified event handle to become set.
    pub async fn wait(&self) {
        return WaitFor {
            shared: &*self.shared,
            waker: &*self.waker,
        }
        .await;

        struct WaitFor<'a> {
            shared: &'a Shared,
            waker: &'a Waker,
        }

        impl Future for WaitFor<'_> {
            type Output = ();

            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                if !self.shared.running.load(Ordering::Acquire) {
                    panic!("background thread panicked");
                }

                if self.waker.ready.load(Ordering::Acquire) {
                    return Poll::Ready(());
                }

                self.waker.waker.register_by_ref(cx.waker());
                Poll::Pending
            }
        }
    }

    /// Set the current event handle.
    pub fn set(&self) {
        self.event.as_ref().unwrap().set();
    }
}

impl RawEvent for AsyncEvent {
    unsafe fn raw_event(&self) -> ss::HANDLE {
        self.event.as_ref().unwrap().raw_event()
    }
}

impl Drop for AsyncEvent {
    fn drop(&mut self) {
        let event = self.event.take().unwrap();
        self.shared.holders.lock().unwrap().removed.push(event);
        self.shared.parker.set();
    }
}

unsafe impl Send for AsyncEvent {}
unsafe impl Sync for AsyncEvent {}
