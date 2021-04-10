use crate::loom::sync::atomic::{AtomicUsize, Ordering};

pub(super) const NONE_READY: usize = 0;
pub(super) const BOTH_READY: usize = 2;

/// The underlying future was not processed in any specific way.
pub(super) const STATE_BUSY: usize = 1;
/// The future has been polled.
pub(super) const STATE_POLLABLE: usize = 2;
/// The task is in a complete state.
pub(super) const STATE_COMPLETE: usize = 4;

/// The state of an async operation.
pub(super) struct State {
    inner: AtomicUsize,
}

impl State {
    /// Construct a new futures state, always starts out as pollable.
    #[inline]
    pub(super) fn new() -> Self {
        Self {
            inner: AtomicUsize::new(STATE_POLLABLE),
        }
    }

    /// Take and mark as busy.
    #[inline]
    pub(super) fn take_busy(&self) -> usize {
        self.inner.swap(STATE_BUSY, Ordering::AcqRel)
    }

    /// Complete the current state.
    #[inline]
    pub(super) fn complete(&self) {
        self.inner.store(STATE_COMPLETE, Ordering::Release);
    }

    /// Test if the state is busy.
    #[inline]
    pub(super) fn is_busy(&self) -> bool {
        self.inner.load(Ordering::Acquire) & STATE_BUSY != 0
    }

    /// Set the current state as pollable.
    #[inline]
    pub(super) fn set_pollable(&self) {
        self.inner.store(STATE_POLLABLE, Ordering::Release);
    }

    /// Mark as pollable and test if busy at the same time.
    #[inline]
    pub(super) fn mark_pollable_and_is_not_busy(&self) -> bool {
        self.inner.fetch_or(STATE_POLLABLE, Ordering::AcqRel) & STATE_BUSY == 0
    }

    /// Unmark as busy and test if pollable.
    #[inline]
    pub(super) fn unmark_busy_and_is_pollable(&self) -> bool {
        self.inner.fetch_and(!STATE_BUSY, Ordering::AcqRel) & STATE_POLLABLE != 0
    }
}
