#[derive(Debug, Clone, Copy)]
pub(super) enum State {
    /// The background thread is busy and cannot process tasks yet. The
    /// scheduler starts out in this state, before the prelude has been
    /// executed.
    Default,
    /// Scheduler is ending.
    End,
}

/// The underlying future was not processed in any specific way.
pub(super) const STATE_BUSY: usize = 1;
/// The future has been polled.
pub(super) const STATE_POLLABLE: usize = 2;
/// The task is in a complete state.
pub(super) const STATE_COMPLETE: usize = 4;

pub(super) const NONE_READY: usize = 0;
pub(super) const BOTH_READY: usize = 2;
