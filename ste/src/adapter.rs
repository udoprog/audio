use crate::tagged::{with_tag, Tag};
use std::future::Future;
use std::pin::Pin;
use std::ptr;
use std::task::{Context, Poll, Waker};

/// Adapter trait used when shipping tasks to the remote thread.
pub(super) trait Adapter: Send {
    /// Poll the adapter, returning a boolean indicating if its complete or not.
    fn poll(&mut self, tag: Tag, waker: &Waker);
}

pub(super) struct FutureAdapter<F>
where
    F: Future,
{
    /// The future being polled.
    future: ptr::NonNull<F>,
    /// Where to store output.
    output: ptr::NonNull<Option<F::Output>>,
}

impl<F> FutureAdapter<F>
where
    F: Future,
{
    pub(super) fn new(future: ptr::NonNull<F>, output: ptr::NonNull<Option<F::Output>>) -> Self {
        Self { future, output }
    }
}

impl<F> Adapter for FutureAdapter<F>
where
    F: Future,
{
    fn poll(&mut self, tag: Tag, waker: &Waker) {
        unsafe {
            let mut cx = Context::from_waker(waker);
            let future = Pin::new_unchecked(self.future.as_mut());

            match with_tag(tag, || future.poll(&mut cx)) {
                Poll::Pending => (),
                Poll::Ready(output) => {
                    *self.output.as_mut() = Some(output);
                }
            }
        }
    }
}

/// Safety: We control how this future is used.
unsafe impl<F> Send for FutureAdapter<F> where F: Future {}
