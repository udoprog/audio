use crate::loom::sync::Mutex;
use crate::state::State;
use std::sync::Arc;
use std::task::{Wake, Waker};

/// Helper structure to transfer a waker.
pub(super) struct SubmitWake {
    pub(super) state: State,
    pub(super) waker: Mutex<Option<Waker>>,
}

impl SubmitWake {
    pub(super) fn inner_wake(&self) {
        if let Some(waker) = &*self.waker.lock().unwrap() {
            waker.wake_by_ref();
        }
    }

    pub(super) unsafe fn release(&self) {
        self.state.complete();

        // Wake up the task so that it sees the panic.
        self.inner_wake();
    }
}

impl Wake for SubmitWake {
    fn wake(self: Arc<Self>) {
        if self.state.mark_pollable_and_is_not_busy() {
            self.inner_wake();
        }
    }
}
