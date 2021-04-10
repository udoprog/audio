use crate::loom::sync::atomic::{AtomicUsize, Ordering};
use crate::loom::sync::Mutex;
use std::sync::Arc;
use std::task::{Wake, Waker};

use crate::state::{STATE_BUSY, STATE_COMPLETE, STATE_POLLABLE};

/// Helper structure to transfer a waker.
pub(super) struct SubmitWake {
    pub(super) state: AtomicUsize,
    pub(super) waker: Mutex<Option<Waker>>,
}

impl SubmitWake {
    pub(super) fn inner_wake(&self) {
        if let Some(waker) = &*self.waker.lock().unwrap() {
            waker.wake_by_ref();
        }
    }

    pub(super) unsafe fn release(&self) {
        self.state.store(STATE_COMPLETE, Ordering::Release);

        // Wake up the task so that it sees the panic.
        if let Some(waker) = &*self.waker.lock().unwrap() {
            waker.wake_by_ref();
        }
    }
}

impl Wake for SubmitWake {
    fn wake(self: Arc<Self>) {
        if self.state.fetch_or(STATE_POLLABLE, Ordering::AcqRel) & STATE_BUSY == 0 {
            self.inner_wake();
        }
    }
}
