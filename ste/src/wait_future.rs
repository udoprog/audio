use crate::linked_list::ListNode;
use crate::state::{STATE_BUSY, STATE_COMPLETE, STATE_POLLABLE};
use crate::submit_wake::SubmitWake;
use crate::worker::{Entry, Shared};
use crate::Panicked;
use std::future::Future;
use std::pin::Pin;
use std::ptr;
use std::sync::atomic::Ordering;
use std::task::{Context, Poll};
use std::thread;

pub(super) struct WaitFuture<'a, T> {
    pub(super) complete: bool,
    pub(super) shared: ptr::NonNull<Shared>,
    pub(super) node: ListNode<Entry>,
    pub(super) output: ptr::NonNull<Option<T>>,
    pub(super) submit_wake: &'a SubmitWake,
    pub(super) thread: Option<&'a thread::Thread>,
}

impl<'a, T> Future for WaitFuture<'a, T> {
    type Output = Result<T, Panicked>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe {
            let this = Pin::get_unchecked_mut(self.as_mut());

            let flags = this.submit_wake.state.swap(STATE_BUSY, Ordering::AcqRel);

            if flags & STATE_COMPLETE != 0 {
                this.complete = true;
            }

            if this.complete {
                return Poll::Ready(Err(Panicked(())));
            }

            if !(flags & STATE_BUSY == 0 && flags & STATE_POLLABLE != 0) {
                return Poll::Pending;
            }

            if let Some(output) = this.output.as_mut().take() {
                this.submit_wake
                    .state
                    .store(STATE_COMPLETE, Ordering::Release);
                return Poll::Ready(Ok(output));
            }

            *this.submit_wake.waker.lock() = Some(cx.waker().clone());

            let first = {
                if let Some(_guard) = this.shared.as_ref().modifier() {
                    this.shared
                        .as_ref()
                        .queue
                        .lock()
                        .push_front(ptr::NonNull::from(&mut this.node))
                } else {
                    return Poll::Ready(Err(Panicked(())));
                }
            };

            if first {
                if let Some(thread) = this.thread {
                    thread.unpark();
                }
            }
        }

        Poll::Pending
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
        while self.submit_wake.state.load(Ordering::Acquire) & STATE_BUSY != 0 {
            thread::yield_now();
        }
    }
}
