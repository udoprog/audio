use parking_lot::{Condvar, Mutex};
use parking_lot_core::{DEFAULT_PARK_TOKEN, DEFAULT_UNPARK_TOKEN};
use std::io;
use std::mem;
use std::sync::Arc;
use std::thread;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("audio thread panicked")]
pub struct Panicked(());

struct Submit {
    /// Where the result of the operation is written back out into. The value
    /// is null as long as the thread has not completed.
    storage: *mut (),
    /// The boxed function which performs the operation.
    task: Box<dyn FnMut(*mut ()) + Send + 'static>,
}

unsafe impl Send for Submit {}

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

impl State {
    /// Drop the state in place in case it is an atomic pointer.
    ///
    /// This prevents the pointer from leaking.
    ///
    /// # Safety
    ///
    /// Caller must ensure that the pointer hasn't already been freed, or is in use.
    ///
    /// Caller also must ensure that the appropriate type is being dropped.
    unsafe fn drop_in_place(&self, drop: unsafe fn(*mut ())) {
        if let State::Submit(submit) = self {
            drop(submit.storage);
        }
    }
}

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
    fn set_state(&self, state: State, drop: unsafe fn(*mut ())) -> Result<(), Panicked> {
        let mut guard = loop {
            let mut guard = self.shared.state.lock();

            match &*guard {
                State::Submit(..) => {
                    self.shared.cond.wait(&mut guard);
                    continue;
                }
                State::Panicked => {
                    // We haven't successfully submitted the state yet, so we
                    // still own the storage pointer.
                    unsafe {
                        state.drop_in_place(drop);
                    }

                    return Err(Panicked(()));
                }
                State::Empty => break guard,
                State::Join => unreachable!(),
            }
        };

        *guard = state;
        Ok(())
    }

    /// Submit a task to run on the background audio thread.
    pub fn submit<F, T>(&self, task: F) -> Result<T, Panicked>
    where
        F: 'static + Send + FnOnce() -> T,
        T: 'static + Send,
    {
        let storage = Box::into_raw(Box::new(None));

        let drop = |ptr: *mut ()| unsafe {
            let _: Box<Option<T>> = Box::from_raw(ptr.cast());
        };

        self.set_state(
            State::Submit(Submit {
                storage: storage.cast(),
                task: Box::new(into_task(task)),
            }),
            drop,
        )?;

        self.handle.thread().unpark();

        // Park until a result is available.
        loop {
            // Try and load storage, if not set yet we continue spinning
            // (spurious wake).
            //
            // Safety: we're the only ones controlling these, so we know that
            // they are correctly allocated and who owns what with
            // synchronization.
            unsafe {
                parking_lot_core::park(
                    storage as usize,
                    || true,
                    || {},
                    |_, _| {},
                    DEFAULT_PARK_TOKEN,
                    None,
                );

                return match *Box::from_raw(storage) {
                    Some(result) => Ok(result),
                    None => Err(Panicked(())),
                };
            }
        }

        fn into_task<F, T>(task: F) -> impl FnMut(*mut ()) + 'static + Send
        where
            F: FnOnce() -> T + 'static + Send,
        {
            let mut task = Some(task);

            return move |storage| {
                // Safety: we know exactly what the underlying type is here.
                let guard: SubmitGuard<T> = SubmitGuard {
                    storage: storage.cast(),
                };

                let task = task.take().expect("task has already been consumed");
                let output = task();
                let mut guard = mem::ManuallyDrop::new(guard);

                // Safety: we're the only one with synchronized access to this
                // pointer, and we know it hasn't been de-allocated yet.
                unsafe {
                    *guard.storage = Some(output);
                }

                guard.unpark();
            };

            struct SubmitGuard<T> {
                storage: *mut Option<T>,
            }

            impl<T> SubmitGuard<T> {
                fn unpark(&self) {
                    loop {
                        // SafetY: the storage address is shared by the entity submitting the task.
                        let result = unsafe {
                            parking_lot_core::unpark_one(self.storage as usize, |_| {
                                DEFAULT_UNPARK_TOKEN
                            })
                        };

                        if result.unparked_threads == 1 {
                            break;
                        }

                        thread::yield_now();
                    }
                }
            }

            impl<T> Drop for SubmitGuard<T> {
                fn drop(&mut self) {
                    self.unpark();
                }
            }
        }
    }

    /// Join the audio background thread.
    pub fn join(self) -> Result<(), Panicked> {
        self.set_state(State::Join, |_| {
            // Woops, pointer leaked. Not much to do about that unfortunately?
        })?;
        self.handle.thread().unpark();
        self.handle.join().map_err(|_| Panicked(()))
    }

    /// Worker thread.
    fn worker(shared: Arc<Shared>) {
        #[cfg(windows)]
        if let Err(e) = windows::initialize_mta() {
            panic!("windows: failed to initialize windows mta: {}", e);
        }

        let shared = PoisonGuard(&shared);

        loop {
            let state = loop {
                thread::park();
                let mut state = shared.0.state.lock();

                match mem::replace(&mut *state, State::Empty) {
                    State::Empty => continue,
                    state => break state,
                }
            };

            match state {
                State::Submit(mut submit) => {
                    (submit.task)(submit.storage);
                    shared.0.cond.notify_one();
                    continue;
                }
                _ => break,
            }
        }

        mem::forget(shared);

        /// Guard used to mark the state of the executed as "panicked". This is
        /// accomplished by asserting that the only reason this destructor would
        /// be called would be due to an unwinding panic.
        struct PoisonGuard<'a>(&'a Shared);

        impl Drop for PoisonGuard<'_> {
            fn drop(&mut self) {
                let mut guard = self.0.state.lock();
                let old = std::mem::replace(&mut *guard, State::Panicked);
                // It's not possible for the state to be anything but empty
                // here, because the worker thread takes the state as one of the
                // first actions it performs.
                debug_assert!(matches!(old, State::Empty));
                self.0.cond.notify_all();
            }
        }
    }
}
