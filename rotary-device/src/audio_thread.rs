use parking_lot::{Condvar, Mutex};
use parking_lot_core::{DEFAULT_PARK_TOKEN, DEFAULT_UNPARK_TOKEN};
use std::io;
use std::mem;
use std::ptr;
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
    shared: ptr::NonNull<Shared>,
    /// The handle associated with the audio thread.
    handle: Option<thread::JoinHandle<()>>,
}

/// Safety: The audio thread is both send and sync because it joins the background
/// thread which keeps track of the state of `shared` and cleans it up once it's
/// no longer needed.
unsafe impl Send for AudioThread {}
unsafe impl Sync for AudioThread {}

impl AudioThread {
    /// Construct a new background audio thread.
    pub fn new() -> io::Result<Self> {
        let shared = ptr::NonNull::from(Box::leak(Box::new(Shared {
            state: Mutex::new(State::Empty),
            cond: Condvar::new(),
        })));

        let shared2 = RawSend(shared);

        let handle = thread::Builder::new()
            .name(String::from("audio-thread"))
            .spawn(move || Self::worker(shared2))?;

        Ok(Self {
            shared,
            handle: Some(handle),
        })
    }

    /// Update the state to the given value.
    fn set_state(&self, state: State) -> Result<(), Panicked> {
        let mut guard = loop {
            unsafe {
                let mut guard = self.shared.as_ref().state.lock();

                match &*guard {
                    State::Submit(..) => {
                        self.shared.as_ref().cond.wait(&mut guard);
                        continue;
                    }
                    State::Panicked => {
                        return Err(Panicked(()));
                    }
                    State::Empty => break guard,
                    State::Join => unreachable!(),
                }
            }
        };

        *guard = state;
        Ok(())
    }

    /// Submit a task to run on the background audio thread.
    pub fn submit<F, T>(&self, task: F) -> Result<T, Panicked>
    where
        F: Send + FnOnce() -> T,
        T: Send,
    {
        let mut storage = None;
        let mut task = into_task(task, RawSend(ptr::NonNull::from(&mut storage)));

        // Safety: We're constructing a pointer to a local stack location. It
        // will never be null.
        //
        // The transmute is necessary because we're constructing a trait object
        // with a `'static` lifetime.
        let task = unsafe {
            ptr::NonNull::new_unchecked(mem::transmute::<&mut (dyn FnMut() + Send), _>(&mut task))
        };
        self.set_state(State::Submit(Submit(task)))?;

        match self.handle.as_ref() {
            Some(handle) => handle.thread().unpark(),
            None => panic!("missing thread handle"),
        }

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

        fn into_task<F, T>(task: F, storage: RawSend<Option<T>>) -> impl FnMut() + Send
        where
            F: FnOnce() -> T + Send,
            T: Send,
        {
            let mut task = Some(task);

            move || {
                let RawSend(mut storage) = storage;

                let _guard = UnparkGuard(storage.as_ptr() as usize);

                if let Some(task) = task.take() {
                    let output = task();

                    // Safety: we're the only one with access to this pointer,
                    // and we know it hasn't been de-allocated yet.
                    unsafe {
                        *storage.as_mut() = Some(output);
                    }
                }
            }
        }

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

    fn inner_join(&mut self) -> Result<(), Panicked> {
        if let Some(handle) = self.handle.take() {
            self.set_state(State::Join)?;
            handle.thread().unpark();
            return handle.join().map_err(|_| Panicked(()));
        }

        Ok(())
    }

    /// Join the audio background thread.
    pub fn join(mut self) -> Result<(), Panicked> {
        self.inner_join()
    }

    /// Worker thread.
    fn worker(RawSend(shared): RawSend<Shared>) {
        #[cfg(windows)]
        if let Err(e) = windows::initialize_mta() {
            panic!("windows: failed to initialize windows mta: {}", e);
        }

        let poison_guard = PoisonGuard(shared);

        'outer: loop {
            let Submit(mut task) = loop {
                unsafe {
                    if !shared.as_ref().cond.notify_one() {
                        thread::park();
                    }

                    let mut state = shared.as_ref().state.lock();

                    match mem::replace(&mut *state, State::Empty) {
                        State::Empty => continue,
                        State::Submit(submit) => break submit,
                        _ => break 'outer,
                    }
                }
            };

            unsafe { task.as_mut()() };
        }

        // Forget the guard to disarm the panic.
        mem::forget(poison_guard);

        /// Guard used to mark the state of the executed as "panicked". This is
        /// accomplished by asserting that the only reason this destructor would
        /// be called would be due to an unwinding panic.
        struct PoisonGuard(ptr::NonNull<Shared>);

        impl Drop for PoisonGuard {
            fn drop(&mut self) {
                unsafe {
                    let old = mem::replace(&mut *self.0.as_ref().state.lock(), State::Panicked);
                    // It's not possible for the state to be anything but empty
                    // here, because the worker thread takes the state before
                    // executing user code which might panic.
                    debug_assert!(matches!(old, State::Empty));
                    self.0.as_ref().cond.notify_all();
                }
            }
        }
    }
}

impl Drop for AudioThread {
    fn drop(&mut self) {
        // Note: we can safely ignore the result, because it will only error in
        // case the background thread has panicked. At which point we're still
        // free to assume it's no longer using the shared state.
        let _ = self.inner_join();

        // Safety: at this point it's guaranteed that we've synchronized with
        // the thread enough that the shared state can be safely deallocated.
        //
        // The background thread is in one of two states:
        // * It has panicked, which means the shared state will not be used any
        //   longer.
        // * It has successfully been joined in. Which has the same
        //   implications.
        unsafe {
            let _ = Box::from_raw(self.shared.as_ptr());
        }
    }
}

/// Small helper for sending things which are not Send.
struct RawSend<T>(ptr::NonNull<T>);
unsafe impl<T> Send for RawSend<T> {}
