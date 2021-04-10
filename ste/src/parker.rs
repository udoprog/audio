// Copied from the Tokio project
// Url: https://github.com/tokio-rs/tokio/blob/b42f21ec3e212ace25331d0c13889a45769e6006/tokio/src/runtime/park.rs
// Under the MIT License.
//
// See: https://github.com/tokio-rs/tokio/blob/master/LICENSE

use crate::loom::sync::atomic::{AtomicUsize, Ordering};
use crate::loom::sync::{Arc, Condvar, Mutex};
use crate::loom::thread;

const EMPTY: usize = 0;
const PARKED_CONDVAR: usize = 1;
const NOTIFIED: usize = 2;

#[repr(transparent)]
struct State(AtomicUsize);

impl State {
    #[inline]
    fn compare_exchange(&self, current: usize, new: usize) -> Result<usize, usize> {
        self.0
            .compare_exchange(current, new, Ordering::SeqCst, Ordering::SeqCst)
    }

    #[inline]
    fn swap(&self, val: usize) -> usize {
        self.0.swap(val, Ordering::SeqCst)
    }
}

struct Inner {
    /// Avoids entering the park if possible.
    state: State,
    /// Used to coordinate access to the condvar.
    mutex: Mutex<()>,
    /// Condvar to block on.
    condvar: Condvar,
}

#[derive(Clone)]
pub struct Parker {
    inner: Arc<Inner>,
}

impl Parker {
    /// Construct a new parker.
    pub(crate) fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                state: State(AtomicUsize::new(EMPTY)),
                mutex: Mutex::new(()),
                condvar: Condvar::new(),
            }),
        }
    }

    pub(crate) fn park(&self) {
        self.inner.park()
    }

    pub(crate) fn unpark(&self) {
        self.inner.unpark()
    }
}

impl Inner {
    /// Parks the current thread.
    fn park(&self) {
        for _ in 0..3 {
            // If we were previously notified then we consume this notification and
            // return quickly.
            if self.state.compare_exchange(NOTIFIED, EMPTY).is_ok() {
                return;
            }

            thread::yield_now();
        }

        self.park_condvar();
    }

    fn park_condvar(&self) {
        // Otherwise we need to coordinate going to sleep
        let mut m = self.mutex.lock().unwrap();

        match self.state.compare_exchange(EMPTY, PARKED_CONDVAR) {
            Ok(_) => {}
            Err(NOTIFIED) => {
                // We must read here, even though we know it will be `NOTIFIED`.
                // This is because `unpark` may have been called again since we read
                // `NOTIFIED` in the `compare_exchange` above. We must perform an
                // acquire operation that synchronizes with that `unpark` to observe
                // any writes it made before the call to unpark. To do that we must
                // read from the write it made to `state`.
                let old = self.state.swap(EMPTY);
                debug_assert_eq!(old, NOTIFIED, "park state changed unexpectedly");

                return;
            }
            Err(actual) => panic!("inconsistent park state; actual = {}", actual),
        }

        loop {
            m = self.condvar.wait(m).unwrap();

            if self.state.compare_exchange(NOTIFIED, EMPTY).is_ok() {
                // got a notification
                return;
            }

            // spurious wakeup, go back to sleep
        }
    }

    fn unpark(&self) {
        // To ensure the unparked thread will observe any writes we made before
        // this call, we must perform a release operation that `park` can
        // synchronize with. To do that we must write `NOTIFIED` even if `state`
        // is already `NOTIFIED`. That is why this must be a swap rather than a
        // compare-and-swap that returns if it reads `NOTIFIED` on failure.
        match self.state.swap(NOTIFIED) {
            EMPTY => {}    // no one was waiting
            NOTIFIED => {} // already unparked
            PARKED_CONDVAR => self.unpark_condvar(),
            actual => panic!("inconsistent state in unpark; actual = {}", actual),
        }
    }

    fn unpark_condvar(&self) {
        // There is a period between when the parked thread sets `state` to
        // `PARKED` (or last checked `state` in the case of a spurious wake
        // up) and when it actually waits on `cvar`. If we were to notify
        // during this period it would be ignored and then when the parked
        // thread went to sleep it would never wake up. Fortunately, it has
        // `lock` locked at this stage so we can acquire `lock` to wait until
        // it is ready to receive the notification.
        //
        // Releasing `lock` before the call to `notify_one` means that when the
        // parked thread wakes it doesn't get woken only to have to wait for us
        // to release `lock`.
        drop(self.mutex.lock());

        self.condvar.notify_one()
    }
}
