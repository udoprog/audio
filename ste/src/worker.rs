use crate::adapter::Adapter;
use crate::lock_free_stack::LockFreeStack;
use crate::loom::sync::atomic::{AtomicIsize, Ordering};
use crate::parker::Parker;
use crate::submit_wake::SubmitWake;
use crate::tagged::Tag;
use crate::thread;
use std::mem;
use std::ptr;
use std::sync::Arc;

/// The type of the prelude function.
pub(super) type Prelude = dyn Fn() + Send + 'static;

// Shared state between the worker thread and [Thread].
pub(super) struct Shared {
    pub(super) modifiers: AtomicIsize,
    pub(super) queue: LockFreeStack<Entry>,
    pub(super) parker: Parker,
}

impl Shared {
    /// Construct new shared state.
    pub(super) fn new() -> Self {
        Self {
            modifiers: AtomicIsize::new(0),
            queue: LockFreeStack::new(),
            parker: Parker::new(),
        }
    }

    /// Add a worker lock.
    pub(super) fn modifier(&self) -> Option<ModifierGuard<'_>> {
        if self.modifiers.fetch_add(1, Ordering::AcqRel) < 0 {
            self.modifiers.fetch_sub(1, Ordering::AcqRel);
            return None;
        }

        Some(ModifierGuard {
            modifiers: &self.modifiers,
        })
    }

    // Release all shared state.
    unsafe fn release(&self) {
        let modifiers = self.modifiers.fetch_add(isize::MIN, Ordering::AcqRel);

        // It's not possible for the state to be anything but empty
        // here, because the worker thread takes the state before
        // executing user code which might panic.
        debug_assert!(modifiers >= 0);

        // NB: we have to wait until the number of modifiers of the queue
        // reaches zero before we can drain it.
        while self.modifiers.load(Ordering::Acquire) != isize::MIN {
            thread::yield_now();
        }

        while let Some(entry) = self.queue.pop() {
            match &entry.as_ref().value {
                Entry::Poll(poll) => {
                    poll.submit_wake.as_ref().release();
                }
                Entry::Schedule(schedule) => {
                    schedule.release();
                }
            }
        }
    }
}

pub(super) struct ModifierGuard<'a> {
    pub(super) modifiers: &'a AtomicIsize,
}

impl Drop for ModifierGuard<'_> {
    fn drop(&mut self) {
        self.modifiers.fetch_sub(1, Ordering::Release);
    }
}

/// Worker thread.
pub(super) fn run(prelude: Option<Box<Prelude>>, shared: ptr::NonNull<Shared>) {
    unsafe {
        let shared = shared.as_ref();

        if let Some(prelude) = prelude {
            let guard = PoisonGuard { shared };
            prelude();
            mem::forget(guard);
        }

        while let Some(m) = shared.modifier() {
            let entry = shared.queue.pop();
            drop(m);

            let mut entry = if let Some(entry) = entry {
                entry
            } else {
                shared.parker.park();
                continue;
            };

            let tag = Tag(shared as *const _ as usize);

            match &mut entry.as_mut().value {
                Entry::Poll(poll) => {
                    let submit_wake = poll.submit_wake.as_ref();

                    let guard = WakerPoisonGuard {
                        shared,
                        submit_wake,
                        parker: poll.parker.as_ref(),
                    };

                    let result = poll.adapter.as_mut().poll(tag, submit_wake);
                    mem::forget(guard);

                    if result {
                        // Immediately ready, set as pollable and wake up.
                        submit_wake.state.set_pollable();
                        submit_wake.inner_wake();
                    } else {
                        // Unset the busy flag and poll it if it has been marked
                        // as pollable.
                        if submit_wake.state.unmark_busy_and_is_pollable() {
                            submit_wake.inner_wake();
                        }
                    }

                    poll.parker.as_ref().unpark();
                }
                Entry::Schedule(schedule) => {
                    let guard = SchedulePoisonGuard { shared, schedule };
                    guard.schedule.task.as_mut()(tag);
                    mem::forget(guard);
                    schedule.release();
                }
            }
        }
    }

    /// Guard used to mark the state of the executed as "panicked". This is
    /// accomplished by asserting that the only reason this destructor would
    /// be called would be due to an unwinding panic.
    struct PoisonGuard<'a> {
        shared: &'a Shared,
    }

    impl Drop for PoisonGuard<'_> {
        fn drop(&mut self) {
            unsafe {
                self.shared.release();
            }
        }
    }

    struct WakerPoisonGuard<'a> {
        shared: &'a Shared,
        submit_wake: &'a SubmitWake,
        parker: &'a Parker,
    }

    impl Drop for WakerPoisonGuard<'_> {
        fn drop(&mut self) {
            // Safety: We know that the task holding the flag owns the
            // reference.
            unsafe {
                self.shared.release();
                self.submit_wake.release();
                self.parker.unpark();
            }
        }
    }

    struct SchedulePoisonGuard<'a> {
        shared: &'a Shared,
        schedule: &'a mut ScheduleEntry,
    }

    impl<'a> Drop for SchedulePoisonGuard<'a> {
        fn drop(&mut self) {
            // Safety: We know that the task holding the flag owns the
            // reference.
            unsafe {
                self.shared.release();
                self.schedule.release();
            }
        }
    }
}

/// An entry onto the task queue.
pub(super) enum Entry {
    /// An entry to immediately be scheduled.
    Schedule(ScheduleEntry),
    /// An entry to be polled.
    Poll(PollEntry),
}

/// A task submitted to the executor.
pub(super) struct ScheduleEntry {
    pub(super) task: ptr::NonNull<dyn FnMut(Tag) + Send + 'static>,
    pub(super) parker: ptr::NonNull<Parker>,
}

impl ScheduleEntry {
    pub(super) fn release(&self) {
        unsafe {
            self.parker.as_ref().unpark();
        }
    }
}

/// A task to be polled by the scheduler.
pub(super) struct PollEntry {
    /// Polling adapter to poll.
    pub(super) adapter: ptr::NonNull<dyn Adapter + 'static>,
    /// Waker to use.
    pub(super) submit_wake: ptr::NonNull<Arc<SubmitWake>>,
    /// Parker to wake once a poll completes.
    pub(super) parker: ptr::NonNull<Parker>,
}
