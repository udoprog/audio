use core::future::Future;
use core::pin::Pin;
use core::ptr::NonNull;
use core::task::{Context, Poll, Waker};

use crate::misc::RawSend;
use crate::parker::Parker;
use crate::tag::{Tag, with_tag};
use crate::worker::{Entry, Shared};

pub(super) struct WaitFuture<'a, F>
where
    F: Future,
{
    /// The future being polled.
    pub(super) future: NonNull<F>,
    /// Where to store output.
    pub(super) output: NonNull<Option<F::Output>>,
    pub(super) parker: NonNull<Parker>,
    pub(super) complete: bool,
    pub(super) shared: &'a Shared,
}

impl<F> Future for WaitFuture<'_, F>
where
    F: Future,
{
    type Output = F::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe {
            let this = Pin::get_unchecked_mut(self.as_mut());

            if this.complete {
                panic!("task already completed");
            }

            let mut task = into_task(
                RawSend::new((&mut this.complete).into()),
                RawSend::new(this.future),
                RawSend::new(this.output),
                RawSend::new(cx.waker().into()),
            );
            let entry = Entry::new(&mut task, this.parker);

            this.shared.schedule_in_place(this.parker, entry);

            if this.complete {
                panic!("background thread panicked");
            }

            if let Some(output) = this.output.as_mut().take() {
                this.complete = true;
                Poll::Ready(output)
            } else {
                Poll::Pending
            }
        }
    }
}

unsafe impl<F> Send for WaitFuture<'_, F> where F: Future {}

fn into_task<F>(
    mut complete: RawSend<bool>,
    mut future: RawSend<F>,
    mut output: RawSend<Option<F::Output>>,
    waker: RawSend<Waker>,
) -> impl FnMut(Tag) + Send
where
    F: Future,
{
    use std::panic;

    move |tag| {
        unsafe {
            // Safety: At this point, we know the waker has been
            // replaced by the polling task and can safely deref it into
            // the underlying waker.
            let waker = waker.as_ref();

            let mut cx = Context::from_waker(waker);
            let future = Pin::new_unchecked(future.as_mut());

            let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                if let Poll::Ready(ready) = with_tag(tag, || future.poll(&mut cx)) {
                    *output.as_mut() = Some(ready);
                }
            }));

            if result.is_err() {
                *complete.as_mut() = true;
            }
        }
    }
}
