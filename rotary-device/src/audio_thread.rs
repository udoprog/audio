use parking_lot::{Condvar, Mutex};
use parking_lot_core::{DEFAULT_PARK_TOKEN, DEFAULT_UNPARK_TOKEN};
use std::io;
use std::mem;
use std::ptr;
use std::sync::Arc;
use std::thread;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("audio thread panicked")]
pub struct Panicked(());

/// The state of the executor.
enum State {
    /// Something is being submitted to the scheduler.
    Submit(Submit),
    /// Scheduler is empty.
    Empty,
    /// Scheduler has panicked.
    Panicked,
    /// Scheduler is being joined.
    Join,
}

/// A task submitted to the executor.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
struct Submit(ptr::NonNull<dyn FnMut() + Send + 'static>);

// The implementation of [Submit] is safe because it's privately constructed
// inside of this module.
unsafe impl Send for Submit {}

struct Shared {
    state: Mutex<State>,
    cond: Condvar,
}

/// Handle to a background audio thread, suitable for running audio-related
/// operations.
///
/// ```rust
/// use std::sync::Arc;
///
/// # fn main() -> anyhow::Result<()> {
/// let audio_thread = Arc::new(rotary_device::AudioThread::new()?);
/// let mut threads = Vec::new();
///
/// for n in 0..10 {
///     let audio_thread = audio_thread.clone();
///
///     threads.push(std::thread::spawn(move || {
///         audio_thread.submit(move || n)
///     }));
/// }
///
/// let mut result = 0;
///
/// for t in threads {
///     result += t.join().unwrap()?;
/// }
///
/// assert_eq!(result, (0..10).sum());
///
/// // Unwrap the audio thread.
/// let audio_thread = Arc::try_unwrap(audio_thread).map_err(|_| "unwrap failed").unwrap();
///
/// let value = audio_thread.submit(|| {
///     panic!("Audio thread: {:?}", std::thread::current().id());
/// });
///
/// println!("Main thread: {:?}", std::thread::current().id());
/// assert!(value.is_err());
///
/// assert!(audio_thread.join().is_err());
/// Ok(())
/// # }
/// ```
#[must_use = "The audio thread should be joined with AudioThread::join once no longer used"]
pub struct AudioThread {
    /// Things that have been submitted for execution on the audio thread.
    shared: Arc<Shared>,
    /// The handle associated with the audio thread.
    handle: thread::JoinHandle<()>,
}

impl AudioThread {
    /// Construct a new background audio thread.
    pub fn new() -> io::Result<Self> {
        let shared = Arc::new(Shared {
            state: Mutex::new(State::Empty),
            cond: Condvar::new(),
        });

        let shared2 = shared.clone();

        let handle = thread::Builder::new()
            .name(String::from("audio-thread"))
            .spawn(move || Self::worker(shared2))?;

        Ok(Self { shared, handle })
    }

    /// Update the state to the given value.
    fn set_state(&self, state: State) -> Result<(), Panicked> {
        let mut guard = loop {
            let mut guard = self.shared.state.lock();

            match &*guard {
                State::Submit(..) => {
                    self.shared.cond.wait(&mut guard);
                    continue;
                }
                State::Panicked => {
                    return Err(Panicked(()));
                }
                State::Empty => break guard,
                State::Join => unreachable!(),
            }
        };

        *guard = state;
        self.handle.thread().unpark();
        Ok(())
    }

    /// Submit a task to run on the background audio thread.
    pub fn submit<F, T>(&self, task: F) -> Result<T, Panicked>
    where
        F: 'static + Send + FnOnce() -> T,
        T: 'static + Send,
    {
        let mut storage = None;
        let mut task = into_task(task, Storage(&mut storage));

        // Safety: We're constructing a pointer to a local stack location. It
        // will never be null.
        let task = unsafe { ptr::NonNull::new_unchecked(&mut task as *mut _) };
        self.set_state(State::Submit(Submit(task)))?;

        // Safety: we're the only ones controlling these, so we know that
        // they are correctly allocated and who owns what with
        // synchronization.
        unsafe {
            parking_lot_core::park(
                &mut storage as *mut _ as usize,
                || true,
                || {},
                |_, _| {},
                DEFAULT_PARK_TOKEN,
                None,
            );
        }

        return match storage {
            Some(result) => Ok(result),
            None => Err(Panicked(())),
        };

        fn into_task<F, T>(task: F, storage: Storage<T>) -> impl FnMut() + 'static + Send
        where
            F: 'static + FnOnce() -> T + Send,
            T: 'static + Send,
        {
            let mut task = Some(task);

            move || {
                let Storage(storage) = storage;

                let _guard = UnparkGuard(storage as usize);

                if let Some(task) = task.take() {
                    let output = task();

                    // Safety: we're the only one with access to this pointer,
                    // and we know it hasn't been de-allocated yet.
                    unsafe {
                        *storage = Some(output);
                    }
                }
            }
        }

        struct Storage<T>(*mut Option<T>);
        unsafe impl<T> Send for Storage<T> where T: Send {}

        /// Guard that guarantees that the given key will be unparked.
        struct UnparkGuard(usize);

        impl Drop for UnparkGuard {
            fn drop(&mut self) {
                loop {
                    // SafetY: the storage address is shared by the entity submitting the task.
                    let result =
                        unsafe { parking_lot_core::unpark_one(self.0, |_| DEFAULT_UNPARK_TOKEN) };

                    if result.unparked_threads == 1 {
                        break;
                    }

                    thread::yield_now();
                }
            }
        }
    }

    /// Join the audio background thread.
    pub fn join(self) -> Result<(), Panicked> {
        self.set_state(State::Join)?;
        self.handle.join().map_err(|_| Panicked(()))
    }

    /// Worker thread.
    fn worker(shared: Arc<Shared>) {
        #[cfg(windows)]
        if let Err(e) = windows::initialize_mta() {
            panic!("windows: failed to initialize windows mta: {}", e);
        }

        let poison_guard = PoisonGuard(&shared);

        'outer: loop {
            let Submit(mut task) = loop {
                if !poison_guard.0.cond.notify_one() {
                    thread::park();
                }

                let mut state = poison_guard.0.state.lock();

                match mem::replace(&mut *state, State::Empty) {
                    State::Empty => continue,
                    State::Submit(submit) => break submit,
                    _ => break 'outer,
                }
            };

            unsafe { task.as_mut()() };
        }

        // Forget the guard to disarm the panic.
        mem::forget(poison_guard);

        /// Guard used to mark the state of the executed as "panicked". This is
        /// accomplished by asserting that the only reason this destructor would
        /// be called would be due to an unwinding panic.
        struct PoisonGuard<'a>(&'a Shared);

        impl Drop for PoisonGuard<'_> {
            fn drop(&mut self) {
                let old = mem::replace(&mut *self.0.state.lock(), State::Panicked);
                // It's not possible for the state to be anything but empty
                // here, because the worker thread takes the state before
                // executing user code which might panic.
                debug_assert!(matches!(old, State::Empty));
                self.0.cond.notify_all();
            }
        }
    }
}
