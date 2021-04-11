use crate::lock_free_stack::Node;
use crate::parker::Parker;
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
}

impl<'a, T> Future for WaitFuture<'a, T> {
    type Output = Result<T, Panicked>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe {
            let this = Pin::get_unchecked_mut(self.as_mut());

            if this.complete {
                return Poll::Ready(Err(Panicked(())));
            }

            // NB: smuggle the current waker in for the duration of the poll.
            if let Entry::Poll(poll) = &mut this.node.value {
                poll.waker = cx.waker() as *const _;
            }

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

            if let Some(output) = this.output.as_mut().take() {
                this.complete = true;
                return Poll::Ready(Ok(output));
            }

            Poll::Pending
        }
    }
}

unsafe impl<T> Send for WaitFuture<'_, T> where T: Send {}
