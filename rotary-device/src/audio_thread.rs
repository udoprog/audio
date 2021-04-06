use parking_lot::{Condvar, Mutex};
use std::io;
use std::marker;
use std::mem;
use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::Arc;
use std::thread;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("audio thread panicked")]
pub struct Panicked(());

struct Submit {
    /// The thread to be woken up once the operation is completed.
    thread: thread::Thread,
    /// Where the result of the operation is written back out into. The value
    /// is null as long as the thread has not completed.
    storage: *mut AtomicPtr<()>,
    /// The boxed function which performs the operation.
    task: Box<dyn FnMut(thread::Thread, *mut AtomicPtr<()>) + Send + 'static>,
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
    fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
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
    fn set_state(&self, state: State) -> Result<(), Panicked> {
        let mut guard = loop {
            let mut guard = self.shared.state.lock();

            match &*guard {
                State::Submit(..) => {
                    self.shared.cond.wait(&mut guard);
                    continue;
                }
                State::Panicked => return Err(Panicked(())),
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
        let thread = thread::current();
        let storage = Box::into_raw(Box::new(AtomicPtr::new(ptr::null_mut())));

        self.set_state(State::Submit(Submit {
            thread: thread.clone(),
            storage,
            task: Box::new(into_task(task)),
        }))?;

        self.handle.thread().unpark();

        // Park until a result is available.
        loop {
            thread::park();

            // Try and load storage, if not set yet we continue spinning
            // (spurious wake).
            //
            // Safety: we're the only ones controlling these, so we know that
            // they are correctly allocated and who owns what with
            // synchronization.
            unsafe {
                let result = (*storage).load(Ordering::Acquire);

                if result.is_null() {
                    continue;
                }

                let _ = Box::from_raw(storage);

                return match *Box::from_raw(result as *mut Option<T>) {
                    Some(result) => Ok(result),
                    None => Err(Panicked(())),
                };
            }
        }

        fn into_task<F, T>(
            task: F,
        ) -> impl FnMut(thread::Thread, *mut AtomicPtr<()>) + 'static + Send
        where
            F: FnOnce() -> T + 'static + Send,
        {
            let mut task = Some(task);

            return move |thread, storage| {
                let guard: SubmitGuard<T> = SubmitGuard {
                    thread,
                    storage,
                    _marker: marker::PhantomData,
                };

                let task = task.take().expect("task has already been consumed");
                let output = task();
                let guard = mem::ManuallyDrop::new(guard);

                let output = Box::into_raw(Box::new(Some(output)));

                // Safety: we're the only one with synchronized access to this
                // pointer, and we know it hasn't been de-allocated yet.
                unsafe {
                    (*guard.storage).store(output as *mut (), Ordering::Release);
                }

                guard.thread.unpark();
            };

            struct SubmitGuard<T> {
                thread: thread::Thread,
                storage: *mut AtomicPtr<()>,
                _marker: marker::PhantomData<T>,
            }

            impl<T> Drop for SubmitGuard<T> {
                fn drop(&mut self) {
                    let output = Box::into_raw(Box::new(Option::<T>::None));

                    // Safety: We free the pointer if we are unwinding due to a
                    // panic.
                    //
                    // We know this is safe, because only user-provided code can
                    // panic, and anything surrounding it is panic-safe. We disarm
                    // the guard *after* user-provided code has executed.
                    unsafe {
                        (*self.storage).store(output as *mut (), Ordering::Release);
                    }

                    self.thread.unpark();
                }
            }
        }
    }

    /// Join the audio background thread.
    pub fn join(self) -> Result<(), Panicked> {
        self.set_state(State::Join)?;
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
            thread::park();
            let mut state = shared.0.state.lock();

            if state.is_empty() {
                continue;
            }

            let state = mem::replace(&mut *state, State::Empty);

            match state {
                State::Submit(mut submit) => {
                    (submit.task)(submit.thread, submit.storage);
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
                *guard = State::Panicked;
                self.0.cond.notify_all();
            }
        }
    }
}
