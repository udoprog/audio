use crate::loom::sync::atomic::Ordering;
use crate::state::STATE_BUSY;
use crate::submit_wake::SubmitWake;
use crate::tagged::{with_tag, Tag};
use std::future::Future;
use std::pin::Pin;
use std::ptr;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};

/// Adapter trait used when shipping tasks to the remote thread.
pub(super) trait Adapter: Send {
    /// Poll the adapter, returning a boolean indicating if its complete or not.
    fn poll(&mut self, tag: Tag, submit_wake: &Arc<SubmitWake>) -> bool;
}

pub(super) struct FutureAdapter<F>
where
    F: Future,
{
    /// Where to store output.
    pub(super) output: ptr::NonNull<Option<F::Output>>,
    /// The future being polled.
    pub(super) future: F,
}

impl<F> Adapter for FutureAdapter<F>
where
    F: Future,
{
    fn poll(&mut self, tag: Tag, submit_wake: &Arc<SubmitWake>) -> bool {
        unsafe {
            debug_assert!(submit_wake.state.load(Ordering::Acquire) & STATE_BUSY != 0);

            let waker = Waker::from(submit_wake.clone());
            let mut cx = Context::from_waker(&waker);
            let future = Pin::new_unchecked(&mut self.future);

            match with_tag(tag, || future.poll(&mut cx)) {
                Poll::Pending => false,
                Poll::Ready(output) => {
                    *self.output.as_mut() = Some(output);
                    true
                }
            }
        }
    }
}

/// Safety: We control how this future is used.
unsafe impl<F> Send for FutureAdapter<F> where F: Future {}
