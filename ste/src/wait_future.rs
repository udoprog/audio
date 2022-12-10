use crate::misc::RawSend;
use crate::parker::Parker;
use crate::tag::{with_tag, Tag};
use crate::worker::{Entry, Shared};
use std::future::Future;
use std::pin::Pin;
use std::ptr;
use std::task::{Context, Poll, Waker};

pub(super) struct WaitFuture<'a, F>
where
    F: Future,
{
    /// The future being polled.
    pub(super) future: ptr::NonNull<F>,
    /// Where to store output.
    pub(super) output: ptr::NonNull<Option<F::Output>>,
    pub(super) parker: ptr::NonNull<Parker>,
    pub(super) complete: bool,
    pub(super) shared: &'a Shared,
}

impl<'a, F> Future for WaitFuture<'a, F>
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
                RawSend((&mut this.complete).into()),
                RawSend(this.future),
                RawSend(this.output),
                RawSend(cx.waker().into()),
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
        let _ = (&waker, &future, &complete);
        unsafe {
            // Safety: At this point, we know the waker has been
            // replaced by the polling task and can safely deref it into
            // the underlying waker.
            let waker = waker.0.as_ref();

            let mut cx = Context::from_waker(waker);
            let future = Pin::new_unchecked(future.0.as_mut());

            let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                let _ = &output;
                if let Poll::Ready(ready) = with_tag(tag, || future.poll(&mut cx)) {
                    *output.0.as_mut() = Some(ready);
                }
            }));

            if result.is_err() {
                *complete.0.as_mut() = true;
            }
        }
    }
}
