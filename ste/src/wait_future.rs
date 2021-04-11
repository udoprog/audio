use crate::lock_free_stack::Node;
use crate::loom::thread;
use crate::parker::Parker;
use crate::state::{STATE_BUSY, STATE_COMPLETE, STATE_POLLABLE};
use crate::submit_wake::SubmitWake;
use crate::worker::{Entry, Shared};
use crate::Panicked;
use std::future::Future;
use std::pin::Pin;
use std::ptr;
use std::task::{Context, Poll};

pub(super) struct WaitFuture<'a, T> {
    pub(super) parker: &'a Parker,
    pub(super) complete: bool,
    pub(super) shared: &'a Shared,
    pub(super) node: Node<Entry>,
    pub(super) output: ptr::NonNull<Option<T>>,
    pub(super) submit_wake: &'a SubmitWake,
}

impl<'a, T> Future for WaitFuture<'a, T> {
    type Output = Result<T, Panicked>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe {
            let this = Pin::get_unchecked_mut(self.as_mut());

            if this.complete {
                return Poll::Ready(Err(Panicked(())));
            }

            let flags = this.submit_wake.state.take_busy();

            if flags & STATE_COMPLETE != 0 {
                this.complete = true;
                return Poll::Ready(Err(Panicked(())));
            }

            if !(flags & STATE_BUSY == 0 && flags & STATE_POLLABLE != 0) {
                return Poll::Pending;
            }

            if let Some(output) = this.output.as_mut().take() {
                this.submit_wake.state.complete();
                return Poll::Ready(Ok(output));
            }

            this.submit_wake.register(cx.waker());

            let first = {
                if let Some(_guard) = this.shared.modifier() {
                    this.shared.queue.push(ptr::NonNull::from(&mut this.node))
                } else {
                    return Poll::Ready(Err(Panicked(())));
                }
            };

            if first {
                this.shared.parker.unpark();
            }

            // NB: We must park here until the remote task wakes us up to allow
            // the task to access things from the environment in the other
            // thread safely.
            //
            // We also know fully that the parker is balanced - i.e. there are
            // no sporadic wakes that can happen because we contrl the state of
            // the submitted task exactly above.
            this.parker.park();
            Poll::Pending
        }
    }
}

unsafe impl<T> Send for WaitFuture<'_, T> where T: Send {}

impl<T> Drop for WaitFuture<'_, T> {
    fn drop(&mut self) {
        if self.complete {
            return;
        }

        // NB: We have no choide but to wait for the state of the submit
        // wake to be safe.
        while self.submit_wake.state.is_busy() {
            thread::yield_now();
        }
    }
}
