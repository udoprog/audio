use crate::linked_list::{LinkedList, Node};
use crate::loom::sync::atomic::{AtomicIsize, Ordering};
use crate::loom::sync::Mutex;
use crate::loom::thread;
use crate::parker::Parker;
use crate::tag::Tag;
use std::mem;
use std::ptr;

/// The type of the prelude function.
pub(super) type Prelude = dyn Fn() + Send + 'static;

// Shared state between the worker thread and [Thread].
pub(super) struct Shared {
    modifiers: AtomicIsize,
    queue: Mutex<LinkedList<Entry>>,
    parker: Parker,
}

impl Shared {
    /// Construct new shared state.
    pub(super) fn new() -> Self {
        Self {
            modifiers: AtomicIsize::new(0),
            queue: Mutex::new(LinkedList::new()),
            parker: Parker::new(),
        }
    }

    /// Construct a guard which while held ensures that the system knows someone
    /// is modifying the worker queue.
    ///
    /// Interior cleanup of the worker queue will only be considered complete
    /// once modifiers reaches 0, because otherwise we run the risk of another
    /// thread being in the middle of a modification while we are cleaning up
    /// and we leave that thread in a blocked state.
    pub(super) fn lock_queue(&self) -> Option<ModifierGuard<'_>> {
        let value = self.modifiers.fetch_add(1, Ordering::SeqCst);

        if value == isize::MAX {
            // We don't have much choice here. Wrapping around is very unlikely
            // to happen because the number of threads required for it to happen
            // is so big. We eprintln in an attempt to get some information out
            // but we really need to abort to maintain the safety of the system.
            eprintln!("ste: modifiers invariant breached, aborting");
            std::process::abort();
        }

        if value < 0 {
            self.modifiers.fetch_sub(1, Ordering::SeqCst);
            return None;
        }

        Some(ModifierGuard {
            modifiers: &self.modifiers,
        })
    }

    // Release all shared state, this will hang until the number of modifiers is
    // zero, after which it will pop all elements from the queue and release
    // them.
    unsafe fn panic_join(&self) {
        let modifiers = self.modifiers.fetch_add(isize::MIN, Ordering::SeqCst);

        // It's not possible for the state to be anything but empty
        // here, because the worker thread takes the state before
        // executing user code which might panic.
        debug_assert!(modifiers >= 0);

        let mut local = self.queue.lock().unwrap().steal();
        release_local_queue(&mut local);

        while self.modifiers.load(Ordering::Acquire) != isize::MIN {
            thread::yield_now();
        }

        let mut local = self.queue.lock().unwrap().steal();
        release_local_queue(&mut local);
    }

    /// Process the given entry on the remote thread.
    ///
    /// # Safety
    ///
    /// We're sending the entry to be executed on a remote thread, the caller
    /// must assure that anything being referenced in it is owned by the caller
    /// and will not be dropped or deallocated for the duration of this call.
    pub(super) unsafe fn schedule_in_place(&self, parker: ptr::NonNull<Parker>, entry: Entry) {
        let mut node = Node::new(entry);

        let first = {
            let _guard = match self.lock_queue() {
                Some(guard) => guard,
                None => panic!("background thread ended"),
            };

            self.queue
                .lock()
                .unwrap()
                .push_front(ptr::NonNull::from(&mut node))
        };

        if first {
            self.parker.unpark();
        }

        // NB: We must park here until the remote task wakes us up to allow
        // the task to access things from the environment in the other
        // thread safely.
        //
        // We also know fully that the parker is balanced - i.e. there are
        // no sporadic wakes that can happen because we contrl the state of
        // the submitted task exactly above.
        parker.as_ref().park();
    }

    /// What should happen when the shared state is joined.
    ///
    /// We mark the modifiers count as negative to signal any entering threads
    /// that they are no longer permitted to push tasks onto the task set.
    pub(super) fn outer_join(&self) {
        // We get the thread to shut down by disallowing the queue to be
        // modified. If the thread has already shut down (due to a panic)
        // this will already have been set to `isize::MIN` and will wrap
        // around or do some other nonsense we can ignore.
        self.modifiers.fetch_add(isize::MIN, Ordering::SeqCst);
        self.parker.unpark();
    }
}

pub(super) struct ModifierGuard<'a> {
    modifiers: &'a AtomicIsize,
}

impl Drop for ModifierGuard<'_> {
    fn drop(&mut self) {
        self.modifiers.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Worker thread.
pub(super) fn run(prelude: Option<Box<Prelude>>, shared: ptr::NonNull<Shared>) {
    unsafe {
        let shared = shared.as_ref();
        let tag = Tag(shared as *const _ as usize);

        if let Some(prelude) = prelude {
            let guard = PoisonGuard { shared };
            prelude();
            mem::forget(guard);
        }

        while let Some(guard) = shared.lock_queue() {
            let mut local = shared.queue.lock().unwrap().steal();
            drop(guard);

            if local.is_empty() {
                shared.parker.park();
                continue;
            }

            while let Some(mut entry) = local.pop_front() {
                let entry = &mut entry.as_mut().value;
                entry.task.as_mut()(tag);
                entry.parker.as_ref().unpark();
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
                self.shared.panic_join();
            }
        }
    }
}

/// A task submitted to the executor.
#[derive(Debug)]
pub(super) struct Entry {
    task: ptr::NonNull<dyn FnMut(Tag) + Send + 'static>,
    parker: ptr::NonNull<Parker>,
}

impl Entry {
    pub(super) unsafe fn new(
        task: &mut (impl FnMut(Tag) + Send),
        parker: ptr::NonNull<Parker>,
    ) -> Self {
        Self {
            task: ptr::NonNull::new_unchecked(mem::transmute::<&mut (dyn FnMut(Tag) + Send), _>(
                task,
            )),
            parker,
        }
    }

    /// Release all resources associated with the entry.
    unsafe fn release(&self) {
        self.parker.as_ref().unpark();
    }
}

/// Helper function to release a local queue.
///
/// This is useful when a queue is stolen, because it disassociates the stolen
/// part of the queue from the rest.
unsafe fn release_local_queue(queue: &mut LinkedList<Entry>) {
    while let Some(entry) = queue.pop_back() {
        entry.as_ref().value.release();
    }
}
