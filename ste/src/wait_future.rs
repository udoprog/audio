use crate::adapter::Adapter;
use crate::parker::Parker;
use crate::worker::{Entry, PollEntry, Shared};
use crate::Panicked;
use std::future::Future;
use std::pin::Pin;
use std::ptr;
use std::task::{Context, Poll};

pub(super) struct WaitFuture<'a, T> {
    pub(super) adapter: ptr::NonNull<dyn Adapter + 'static>,
    pub(super) parker: ptr::NonNull<Parker>,
    pub(super) complete: bool,
    pub(super) shared: &'a Shared,
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
            let poll_entry = PollEntry::new(this.adapter, cx.waker().into(), this.parker);

            this.shared
                .schedule_in_place(this.parker, Entry::Poll(poll_entry))?;

            if let Some(output) = this.output.as_mut().take() {
                this.complete = true;
                Poll::Ready(Ok(output))
            } else {
                Poll::Pending
            }
        }
    }
}

unsafe impl<T> Send for WaitFuture<'_, T> where T: Send {}
