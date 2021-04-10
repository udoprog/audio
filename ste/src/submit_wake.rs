use crate::atomic_waker::AtomicWaker;
use crate::state::State;
use std::sync::Arc;
use std::task::{Wake, Waker};

/// Helper structure to transfer a waker.
pub(super) struct SubmitWake {
    pub(super) state: State,
    waker: AtomicWaker,
}

impl SubmitWake {
    pub(super) fn new() -> Self {
        Self {
            state: State::new(),
            waker: AtomicWaker::new(),
        }
    }

    pub(super) fn register(&self, waker: &Waker) {
        self.waker.register(waker);
    }

    pub(super) fn inner_wake(&self) {
        self.waker.wake();
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
