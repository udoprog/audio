//! [![Documentation](https://docs.rs/ste/badge.svg)](https://docs.rs/ste)
//! [![Crates](https://img.shields.io/crates/v/ste.svg)](https://crates.io/crates/ste)
//! [![Actions Status](https://github.com/udoprog/audio/workflows/Rust/badge.svg)](https://github.com/udoprog/audio/actions)
//!
//! A single-threaded executor with some tricks up its sleeve.
//!
//! This was primarily written for use in [audio] as a low-latency way of
//! interacting with a single background thread for audio-related purposes, but
//! is otherwise a general purpose library that can be used by anyone.
//!
//! > **Soundness Warning:** This crate uses a fair bit of **unsafe**. Some of
//! > the tricks employed needs to be rigirously sanity checked for safety
//! > before you can rely on this for production uses.
//!
//! The default way to access the underlying thread is through the [submit]
//! method. This blocks the current thread for the duration of the task allowing
//! the background thread to access variables which are in scope. Like `n`
//! below.
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
//! # Restricting which threads can access data
//!
//! We provide the [Tagged] container. Things stored in this container may
//! *only* be accessed by the thread in which the container was created.
//!
//! It works by associating a tag with the data that is unique to the thread
//! which created it. Any attempt to access the data will check this tag against
//! the tag in the current thread.
//!
//! ```rust,should_panic
//! struct Foo;
//!
//! impl Foo {
//!     fn say_hello(&self) {
//!         println!("Hello World!");
//!     }
//! }
//!
//! # fn main() -> anyhow::Result<()> {
//! let thread = ste::Thread::new()?;
//!
//! let foo = thread.submit(|| ste::Tagged::new(Foo))?;
//! foo.say_hello(); // <- Panics!
//!
//! thread.join()?;
//! # Ok(()) }
//! ```
//!
//! Using it inside of the thread that created it is fine.
//!
//! ```rust
//! # struct Foo;
//! # impl Foo { fn say_hello(&self) { println!("Hello World!"); } }
//! # fn main() -> anyhow::Result<()> {
//! let thread = ste::Thread::new()?;
//!
//! let foo = thread.submit(|| ste::Tagged::new(Foo))?;
//!
//! thread.submit(|| {
//!     foo.say_hello(); // <- OK!
//! })?;
//!
//! thread.join()?;
//! # Ok(()) }
//! ```
//!
//! > There are some other details you need to know relevant to how to use the
//! > [Tagged] container. See its documentation for more.
//!
//! # Known unsafety and soundness issues
//!
//! Below you can find a list of known soundness issues this library currently
//! has.
//!
//! ## Pointers to stack-local addresses
//!
//! In order to efficiently share data between a thread calling [submit] and the
//! background thread, the background thread references a fair bit of
//! stack-local data from the calling thread which involves a fair bit of
//! `unsafe`.
//!
//! While it should be possible to make this use *safe* (as is the hope of this
//! library), it carries a risk that if the background thread were to proceed
//! executing a task that is no longer synchronized properly with a caller of
//! [submit] it might end up referencing data which is either no longer valid
//! (use after free), or contains something else (dirty).
//!
//! ## Soundness issue with tag re-use
//!
//! [Tagged] containers currently use a tag based on the address of a slab of
//! allocated memory that is associated with each [Thread]. If however a
//! [Thread] is shut down, and a new later recreated, there is a slight risk
//! that this might re-use an existing memory address.
//!
//! Memory addresses are quite thankful to use, because they're cheap and quite
//! easy to access. Due to this it might however be desirable to use a generated
//! ID per thread instead which can for example abort a program in case it can't
//! guarantee uniqueness.
//!
//! [submit]: https://docs.rs/ste/0/ste/struct.Thread.html#method.submit
//! [Thread]: https://docs.rs/ste/0/ste/struct.Thread.html
//! [Tagged]: https://docs.rs/ste/0/ste/struct.Tagged.html
//! [audio]: https://github.com/udoprog/audio

use parking_lot::Mutex;
use std::future::Future;
use std::io;
use std::mem;
use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use thiserror::Error;

#[cfg(test)]
mod tests;

mod parker;

mod worker;
use self::worker::{Entry, PollEntry, Prelude, ScheduleEntry, Shared};

mod tagged;
pub use self::tagged::Tagged;
use self::tagged::{with_tag, Tag};

#[doc(hidden)]
pub mod linked_list;

#[doc(hidden)]
pub mod lock_free_stack;
use self::lock_free_stack::Node;

mod submit_wake;
use self::submit_wake::SubmitWake;

mod state;
use self::state::{BOTH_READY, NONE_READY, STATE_POLLABLE};

mod adapter;
use self::adapter::{Adapter, FutureAdapter};

mod wait_future;
use self::wait_future::WaitFuture;

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

    /// Run the given future on the background thread. The future can reference
    /// memory outside of the current scope, but will cause the runtime to block
    /// if it's being dropped until completion.
    ///
    /// # Safety
    ///
    /// This function is unsafe as heck right now. Polling it without it having
    /// been called w/ wake_by_ref **will cause a data race**.
    ///
    /// The above will be fixed.
    ///
    /// # Examples
    ///
    /// This method supports panics the same way as other threads:
    ///
    /// ```rust
    /// # #[tokio::main(flavor = "current_thread")] async fn main() -> anyhow::Result<()> {
    /// let audio_thread = ste::Thread::new()?;
    ///
    /// let result = audio_thread
    ///     .submit_async(async move { panic!("woops") })
    ///     .await;
    ///
    /// assert!(result.is_err());
    /// assert!(audio_thread.join().is_err());
    /// # Ok(()) }
    /// ```
    pub async fn submit_async<F>(&self, future: F) -> Result<F::Output, Panicked>
    where
        F: Send + Future,
        F::Output: Send,
    {
        // Stack location where the output of the compuation is stored.
        let mut output = None;

        // The state of the thing being polled.
        let submit_wake = Arc::new(SubmitWake {
            state: AtomicUsize::new(STATE_POLLABLE),
            waker: Mutex::new(None),
        });

        let mut adapter = FutureAdapter {
            future,
            output: ptr::NonNull::from(&mut output),
        };

        let wait_future = WaitFuture {
            complete: false,
            shared: self.shared,
            node: Node::new(Entry::Poll(PollEntry {
                adapter: ptr::NonNull::from(unsafe {
                    let adapter: &mut dyn Adapter = &mut adapter;
                    mem::transmute::<_, &mut dyn Adapter>(adapter)
                }),
                submit_wake: ptr::NonNull::from(&submit_wake),
            })),
            output: ptr::NonNull::from(&mut output),
            submit_wake: &*submit_wake,
            thread: self.handle.as_ref().map(|h| h.thread()),
        };

        wait_future.await
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
        let flag = AtomicUsize::new(0);
        let mut storage = None;

        {
            let storage = ptr::NonNull::from(&mut storage);
            let (parker, unparker) = parker::new(storage.as_ptr());

            let mut task = into_task(task, storage);

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

            let mut schedule = Node::new(Entry::Schedule(ScheduleEntry {
                task,
                unparker,
                flag: ptr::NonNull::from(&flag),
            }));

            unsafe {
                let first = {
                    let shared = self.shared.as_ref();

                    let _guard = match shared.modifier() {
                        Some(guard) => guard,
                        None => return Err(Panicked(())),
                    };

                    shared.queue.push(ptr::NonNull::from(&mut schedule))
                };

                if first {
                    if let Some(handle) = &self.handle {
                        handle.thread().unpark();
                    }
                }
            }

            // If 0, we know we got here first and have to park until the thread
            // is ready.
            if flag.fetch_add(1, Ordering::AcqRel) == NONE_READY {
                // Safety: we're the only ones controlling these, so we know that
                // they are correctly allocated and who owns what with
                // synchronization.
                parker.park(|| flag.load(Ordering::Relaxed) == BOTH_READY);
            }
        }

        return match storage {
            Some(result) => Ok(result),
            None => Err(Panicked(())),
        };

        fn into_task<T, O>(task: T, storage: ptr::NonNull<Option<O>>) -> impl FnMut(Tag) + Send
        where
            T: FnOnce() -> O + Send,
            O: Send,
        {
            let mut task = Some(task);
            let storage = RawSend(storage);

            move |tag| {
                let RawSend(mut storage) = storage;

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
    }

    /// Move the provided `value` onto the background thread and drop it.
    ///
    /// This is necessary for [Tagged] values that needs to be dropped which
    /// would otherwise panic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Ensure that `Foo` is both `!Send` and `!Sync`.
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
    /// If we omit the call to [drop][Thread::drop], the above will panic.
    ///
    /// ```rust,should_panic
    /// # struct Foo(*mut ());
    /// # impl Drop for Foo { fn drop(&mut self) {} }
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

    fn inner_join(&mut self) -> Result<(), Panicked> {
        if let Some(handle) = self.handle.take() {
            unsafe {
                self.shared
                    .as_ref()
                    .state
                    .fetch_sub(isize::MIN, Ordering::AcqRel);
            }

            handle.thread().unpark();
            return handle.join().map_err(|_| Panicked(()));
        }

        Ok(())
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
        let shared = ptr::NonNull::from(Box::leak(Box::new(Shared::new())));

        let prelude = self.prelude;

        let shared2 = RawSend(shared);

        let handle = thread::Builder::new()
            .name(String::from("ste-thread"))
            .spawn(move || {
                let RawSend(shared) = shared2;
                worker::run(prelude, shared)
            })?;

        Ok(Thread {
            shared,
            handle: Some(handle),
        })
    }
}

/// Small helper for sending things which are not Send.
struct RawSend<T>(ptr::NonNull<T>);
unsafe impl<T> Send for RawSend<T> {}
