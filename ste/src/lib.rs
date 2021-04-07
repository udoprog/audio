//! [![Documentation](https://docs.rs/ste/badge.svg)](https://docs.rs/ste)
//! [![Crates](https://img.shields.io/crates/v/ste.svg)](https://crates.io/crates/ste)
//! [![Actions Status](https://github.com/udoprog/rotary/workflows/Rust/badge.svg)](https://github.com/udoprog/rotary/actions)
//!
//! A single-threaded executor with some tricks up its sleeve.
//!
//! This was primarily written for use in [rotary] as a low-latency way of
//! interacting with a single background thread for audio-related purposes, but
//! is otherwise a general purpose library that can be used by anyone.
//!
//! **Warning:** Some of the tricks used in this crate needs to be sanity
//! checked for safety before you can rely on this for production uses.
//!
//! # Examples
//!
//! ```rust
//! # fn main() -> anyhow::Result<()> {
//! let thread = ste::Thread::new()?;
//!
//! let mut n = 10;
//! thread.submit(|| n += 10)?;
//! assert_eq!(20, n);
//!
//! thread.join()?;
//! # Ok(()) }    
//! ```
//!
//! [rotary]: https://github.com/udoprog/rotary

use parking_lot::{Condvar, Mutex};
use parking_lot_core::{DEFAULT_PARK_TOKEN, DEFAULT_UNPARK_TOKEN};
use std::io;
use std::mem;
use std::ptr;
use std::thread;
use thiserror::Error;

mod tagged;
pub use self::tagged::Tagged;
use self::tagged::{with_tag, Tag};

/// Error raised when we try to interact with a background thread that has
/// panicked.
#[derive(Debug, Error)]
#[error("background thread panicked")]
pub struct Panicked(());

/// The handle for a background thread.
///
/// The background thread can be interacted with in a couple of ways:
/// * [submit][Thread::submit] - for submitted tasks, the call will block until
///   it has been executed on the thread (or the thread has panicked).
/// * [drop][Thread::drop] - for dropping value *on* the background thread. This
///   is necessary for [Tagged] values that requires drop.
///
/// # Examples
///
/// ```rust
/// use std::sync::Arc;
///
/// # fn main() -> anyhow::Result<()> {
/// let thread = Arc::new(ste::Thread::new()?);
/// let mut threads = Vec::new();
///
/// for n in 0..10 {
///     let thread = thread.clone();
///
///     threads.push(std::thread::spawn(move || {
///         thread.submit(move || n)
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
/// // Unwrap the thread.
/// let thread = Arc::try_unwrap(thread).map_err(|_| "unwrap failed").unwrap();
///
/// let value = thread.submit(|| {
///     panic!("Background thread: {:?}", std::thread::current().id());
/// });
///
/// println!("Main thread: {:?}", std::thread::current().id());
/// assert!(value.is_err());
///
/// assert!(thread.join().is_err());
/// # Ok(()) }
/// ```
#[must_use = "The thread should be joined with Thread::join once no longer used, \
    otherwise it will block while being dropped."]
pub struct Thread {
    /// Things that have been submitted for execution on the background thread.
    shared: ptr::NonNull<Shared>,
    /// The handle associated with the background thread.
    handle: Option<thread::JoinHandle<()>>,
}

/// Safety: The handle is both send and sync because it joins the background
/// thread which keeps track of the state of `shared` and cleans it up once it's
/// no longer needed.
unsafe impl Send for Thread {}
unsafe impl Sync for Thread {}

impl Thread {
    /// Construct a default background thread executor.
    ///
    /// These both do the same thing:
    ///
    /// ```rust
    /// # fn main() -> anyhow::Result<()> {
    /// let thread1 = ste::Thread::new()?;
    /// let thread2 = ste::Builder::new().build()?;
    /// # Ok(()) }
    /// ```
    pub fn new() -> io::Result<Self> {
        Builder::new().build()
    }

    /// Submit a task to run on the background thread.
    ///
    /// The call will block until it has been executed on the thread (or the
    /// thread has panicked).
    ///
    /// Because this function blocks until completion, it can safely access
    /// values which are outside of the scope of the provided closure.
    ///
    /// If you however need to store and access things which are `!Sync`, you
    /// can use [Tagged].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> anyhow::Result<()> {
    /// let thread = ste::Thread::new()?;
    ///
    /// let mut n = 10;
    /// thread.submit(|| n += 10)?;
    /// assert_eq!(20, n);
    ///
    /// thread.join()?;
    /// # Ok(()) }    
    /// ```
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
            ptr::NonNull::new_unchecked(mem::transmute::<&mut (dyn FnMut(Tag) + Send), _>(
                &mut task,
            ))
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

        fn into_task<F, T>(task: F, storage: RawSend<Option<T>>) -> impl FnMut(Tag) + Send
        where
            F: FnOnce() -> T + Send,
            T: Send,
        {
            let mut task = Some(task);

            move |tag| {
                let RawSend(mut storage) = storage;

                let _guard = UnparkGuard(storage.as_ptr() as usize);

                if let Some(task) = task.take() {
                    let output = with_tag(tag, task);

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

    /// Move the provided `value` onto the background thread and drop it.
    ///
    /// This is necessary for [Tagged] values that needs to be dropped which
    /// would otherwise panic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Ensure that `Foo` is `!Send` and `!Sync`.
    /// struct Foo(*mut ());
    ///
    /// impl Foo {
    ///     fn test(&self) -> u32 {
    ///         42
    ///     }
    /// }
    ///
    /// impl Drop for Foo {
    ///     fn drop(&mut self) {
    ///         println!("Foo was dropped");
    ///     }
    /// }
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let thread = ste::Thread::new()?;
    ///
    /// let value = thread.submit(|| ste::Tagged::new(Foo(0 as *mut ())))?;
    /// let out = thread.submit(|| value.test())?;
    /// assert_eq!(42, out);
    ///
    /// thread.drop(value)?;
    /// thread.join()?;
    /// # Ok(()) }    
    /// ```
    ///
    /// If we omit the drop the above will panic.
    ///
    /// ```rust,should_panic
    /// # struct Foo(*mut ());
    /// # impl Drop for Foo { fn drop(&mut self) {} }
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let thread = ste::Thread::new()?;
    ///
    /// let value = thread.submit(|| ste::Tagged::new(Foo(0 as *mut ())))?;
    ///
    /// thread.join()?;
    /// # Ok(()) }    
    /// ```
    pub fn drop<T>(&self, value: T) -> Result<(), Panicked>
    where
        T: Send,
    {
        self.submit(move || drop(value))?;
        Ok(())
    }

    /// Join the background thread.
    ///
    /// Will block until the background thread is joined.
    ///
    /// This is the clean way to join a background thread, the alternative is to
    /// let [Thread] drop and this will be performed in the drop handler
    /// instead.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> anyhow::Result<()> {
    /// let thread = ste::Thread::new()?;
    ///
    /// let mut n = 10;
    /// thread.submit(|| n += 10)?;
    /// assert_eq!(20, n);
    ///
    /// thread.join()?;
    /// # Ok(()) }    
    /// ```
    pub fn join(mut self) -> Result<(), Panicked> {
        self.inner_join()
    }

    /// Update the shared state.
    ///
    /// This will *not* notify the background thread to allow it to be used in
    /// contexts where the handle has been removed.
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

    fn inner_join(&mut self) -> Result<(), Panicked> {
        if let Some(handle) = self.handle.take() {
            self.set_state(State::Join)?;
            handle.thread().unpark();
            return handle.join().map_err(|_| Panicked(()));
        }

        Ok(())
    }

    /// Worker thread.
    fn worker(prelude: Option<Box<Prelude>>, RawSend(shared): RawSend<Shared>) {
        if let Some(prelude) = prelude {
            prelude();
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

            let tag = Tag(shared.as_ptr() as usize);
            unsafe { task.as_mut()(tag) };
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

impl Drop for Thread {
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

/// The builder for a [Thread] which can be configured a bit more.
pub struct Builder {
    prelude: Option<Box<Prelude>>,
}

impl Builder {
    /// Construct a new builder.
    pub fn new() -> Self {
        Self { prelude: None }
    }

    /// Configure a prelude to the [Thread]. This is code that will run just as
    /// the thread is spinning up.
    ///
    /// # Examples
    ///
    /// ```rust
    /// fn say_hello(main_thread: std::thread::ThreadId) {
    ///     println!("Hello from the prelude!");
    ///     assert_ne!(main_thread, std::thread::current().id());
    /// }
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let main_thread = std::thread::current().id();
    ///
    /// let thread = ste::Builder::new().prelude(move || say_hello(main_thread)).build();
    /// # Ok(()) }
    /// ```
    pub fn prelude<P>(self, prelude: P) -> Self
    where
        P: Fn() + Send + 'static,
    {
        Self {
            prelude: Some(Box::new(prelude)),
        }
    }

    /// Construct the background thread.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> anyhow::Result<()> {
    /// let thread = ste::Builder::new().build()?;
    /// thread.join()?;
    /// # Ok(()) }
    /// ```
    pub fn build(self) -> io::Result<Thread> {
        let shared = ptr::NonNull::from(Box::leak(Box::new(Shared {
            state: Mutex::new(State::Empty),
            cond: Condvar::new(),
        })));

        let prelude = self.prelude;

        let shared2 = RawSend(shared);

        let handle = thread::Builder::new()
            .name(String::from("ste-thread"))
            .spawn(move || Thread::worker(prelude, shared2))?;

        Ok(Thread {
            shared,
            handle: Some(handle),
        })
    }
}

/// Small helper for sending things which are not Send.
struct RawSend<T>(ptr::NonNull<T>);
unsafe impl<T> Send for RawSend<T> {}

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
struct Submit(ptr::NonNull<dyn FnMut(Tag) + Send + 'static>);

// The implementation of [Submit] is safe because it's privately constructed
// inside of this module.
unsafe impl Send for Submit {}

// Shared state between the worker thread and [Thread].
struct Shared {
    state: Mutex<State>,
    cond: Condvar,
}

/// The type of the prelude function.
type Prelude = dyn Fn() + Send + 'static;
