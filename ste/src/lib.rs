//! [![Documentation](https://docs.rs/ste/badge.svg)](https://docs.rs/ste)
//! [![Crates](https://img.shields.io/crates/v/ste.svg)](https://crates.io/crates/ste)
//! [![Actions Status](https://github.com/udoprog/audio/workflows/Rust/badge.svg)](https://github.com/udoprog/audio/actions)
//!
//! A single-threaded executor with some tricks up its sleeve.
//!
//! This was primarily written for use in [audio] as a low-latency way of
//! interacting with a single background thread for audio-related purposes, but
//! is otherwise a general purpose library that can be used to do anything.
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
//! let thread = ste::spawn();
//!
//! let mut n = 10;
//! thread.submit(|| n += 10);
//! assert_eq!(20, n);
//!
//! thread.join();
//! # Ok(()) }    
//! ```
//!
//! # Restricting thread access using tags
//!
//! This library provides the ability to construct a [Tag] which is uniquely
//! associated with the thread that created it. This can then be used to ensure
//! that data is only accessible by one thread.
//!
//! This is useful, because many APIs requires *thread-locality* where instances
//! can only safely be used by the thread that created them. This is a low-level
//! tool we provide which allows the safe implementation of `Send` for types
//! which are otherwise `!Send`.
//!
//! Note that correctly using a [Tag] is hard, and incorrect use has severe
//! safety implications. Make sure to study its documentation closely before
//! use.
//!
//! ```rust
//! struct Foo {
//!     tag: ste::Tag,
//! }
//!
//! impl Foo {
//!     fn new() -> Self {
//!         Self {
//!             tag: ste::Tag::current_thread(),
//!         }
//!     }
//!
//!     fn say_hello(&self) {
//!         self.tag.ensure_on_thread();
//!         println!("Hello World!");
//!     }
//! }
//!
//! # fn main() -> anyhow::Result<()> {
//! let thread = ste::spawn();
//!
//! let foo = thread.submit(|| Foo::new());
//!
//! thread.submit(|| {
//!     foo.say_hello(); // <- OK!
//! });
//!
//! thread.join();
//! # Ok(()) }
//! ```
//!
//! Using `say_hello` outside of the thread that created it is not fine and will
//! panic to prevent racy access:
//!
//! ```rust,should_panic
//! # struct Foo { tag: ste::Tag }
//! # impl Foo {
//! #     fn new() -> Self { Self { tag: ste::Tag::current_thread() } }
//! #     fn say_hello(&self) { self.tag.ensure_on_thread(); }
//! # }
//! # fn main() -> anyhow::Result<()> {
//! let thread = ste::spawn();
//!
//! let foo = thread.submit(|| Foo::new());
//!
//! foo.say_hello(); // <- Oops, panics!
//!
//! thread.join();
//! # Ok(()) }
//! ```
//!
//! # Known unsafety and soundness issues
//!
//! Below you can find a list of unsafe use and known soundness issues this
//! library currently has. The soundness issues **must be fixed** before this
//! library goes out of *alpha*.
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
//! ## Tag re-use
//!
//! [Tag] containers currently use a tag based on the address of a slab of
//! allocated memory that is associated with each [Thread]. If however a
//! [Thread] is shut down, and a new later recreated, there is a slight risk
//! that this might re-use an existing memory address.
//!
//! Memory addresses are quite thankful to use, because they're cheap and quite
//! easy to access. Due to this it might however be desirable to use a generated
//! ID per thread instead which can for example abort a program in case it can't
//! guarantee uniqueness.
//!
//! [audio]: https://github.com/udoprog/audio
//! [submit]: https://docs.rs/ste/*/ste/struct.Thread.html#method.submit
//! [Tag]: https://docs.rs/ste/*/ste/struct.Tag.html
//! [Thread]: https://docs.rs/ste/*/ste/struct.Thread.html

use std::future::Future;
use std::io;
use std::ptr;

pub(crate) mod loom;
use self::loom::thread;

#[cfg(test)]
mod tests;

mod parker;
use crate::parker::Parker;

mod worker;
use self::worker::{Entry, Prelude, Shared};

mod tag;
use self::tag::with_tag;
pub use self::tag::Tag;

#[doc(hidden)]
pub mod linked_list;

mod wait_future;
use self::wait_future::WaitFuture;

mod misc;
use self::misc::RawSend;

/// Construct a default background thread executor.
///
/// These both do the same thing, except the builder allows you to catch an OS error:
///
/// ```rust
/// # fn main() -> anyhow::Result<()> {
/// let thread1 = ste::spawn();
/// let thread2 = ste::Builder::new().build()?;
/// # Ok(()) }
/// ```
pub fn spawn() -> Thread {
    Builder::new().build().expect("failed to spawn thread")
}

/// The handle for a background thread.
///
/// The background thread can be interacted with in a couple of ways:
/// * [submit][Thread::submit] - for submitted tasks, the call will block until
///   it has been executed on the thread (or the thread has panicked).
/// * [submit_async][Thread::submit_async] - for submitting asynchronous tasks,
///   the call will block until it has been executed on the thread (or the
///   thread has panicked).
/// * [drop][Thread::drop] - for dropping value *on* the background thread. This
///   is necessary for [Tag] values that requires drop.
///
/// # Tasks panicking
///
/// If anything on the background thread ends up panicking, the panic will be
/// propagated but also isolated to that one task.
///
/// Note that this is only true for unwinding panics. It would not apply to
/// panics resulting in aborts.
///
/// # Examples
///
/// ```rust
/// use std::sync::Arc;
/// use std::panic::{AssertUnwindSafe, catch_unwind};
///
/// # fn main() -> anyhow::Result<()> {
/// let thread = Arc::new(ste::spawn());
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
///     result += t.join().unwrap();
/// }
///
/// assert_eq!(result, (0..10).sum());
///
/// // Unwrap the thread.
/// let thread = Arc::try_unwrap(thread).map_err(|_| "unwrap failed").unwrap();
///
/// let result = catch_unwind(AssertUnwindSafe(|| thread.submit(|| {
///     panic!("Background thread: {:?}", std::thread::current().id());
/// })));
///
/// assert!(result.is_err());
///
/// println!("Main thread: {:?}", std::thread::current().id());
///
/// thread.join();
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
    /// Submit a task to run on the background thread.
    ///
    /// The call will block until it has been executed on the thread (or the
    /// thread has panicked).
    ///
    /// Because this function blocks until completion, it can safely access
    /// values which are outside of the scope of the provided closure.
    ///
    /// If you however need to store and access things which are `!Send`, you
    /// can wrap them in a container that ensures their thread-locality with
    /// [Tag] and then safely implement [Send] for it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> anyhow::Result<()> {
    /// let thread = ste::spawn();
    ///
    /// let mut n = 10;
    /// thread.submit(|| n += 10);
    /// assert_eq!(20, n);
    ///
    /// thread.join();
    /// # Ok(()) }    
    /// ```
    ///
    /// Unwinding panics as isolated on a per-task basis.
    ///
    /// ```rust
    /// use std::panic::{AssertUnwindSafe, catch_unwind};
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> anyhow::Result<()> {
    /// let thread = ste::spawn();
    ///
    /// let result = catch_unwind(AssertUnwindSafe(|| thread.submit(|| panic!("woops"))));
    /// assert!(result.is_err());
    ///
    /// let mut result = 0;
    /// thread.submit(|| { result += 1 });
    /// assert_eq!(result, 1);
    ///
    /// thread.join();
    /// # Ok(()) }
    /// ```
    pub fn submit<F, T>(&self, task: F) -> T
    where
        F: Send + FnOnce() -> T,
        T: Send,
    {
        unsafe {
            let mut storage = None;
            let parker = Parker::new();

            let mut task = into_task(task, RawSend(ptr::NonNull::from(&mut storage)));
            let entry = Entry::new(&mut task, ptr::NonNull::from(&parker));

            // Safety: We're constructing a pointer to a local stack location. It
            // will never be null.
            //
            // The transmute is necessary because we're constructing a trait object
            // with a `'static` lifetime.
            self.shared
                .as_ref()
                .schedule_in_place(ptr::NonNull::from(&parker), entry);

            return match storage {
                Some(result) => result,
                None => panic!("background thread panicked"),
            };
        }

        fn into_task<T, O>(task: T, mut storage: RawSend<Option<O>>) -> impl FnMut(Tag) + Send
        where
            T: FnOnce() -> O + Send,
            O: Send,
        {
            use std::panic;

            let mut task = Some(task);

            move |tag| {
                if let Some(task) = task.take() {
                    let _ = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                        let output = with_tag(tag, task);

                        // Safety: we're the only one with access to this pointer,
                        // and we know it hasn't been de-allocated yet.
                        unsafe {
                            *storage.0.as_mut() = Some(output);
                        }
                    }));
                }
            }
        }
    }

    /// Run the given future on the background thread. The future can reference
    /// memory outside of the current scope, but in order to do so, every time
    /// it is polled it has to be perfectly synchronized with a remote poll
    /// happening on the background thread.
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
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> anyhow::Result<()> {
    /// let thread = ste::Builder::new().with_tokio().build()?;
    ///
    /// thread.submit_async(async {
    ///     tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    ///     println!("Hello World!");
    /// });
    ///
    /// thread.join();
    /// # Ok(()) }
    /// ```
    ///
    /// Unwinding panics as isolated on a per-task basis the same was as for
    /// [submit][Thread::submit].
    ///
    /// ```rust,should_panic
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> anyhow::Result<()> {
    /// let thread = ste::spawn();
    ///
    /// thread.submit_async(async move { panic!("woops") }).await;
    ///
    /// // Note: thread will still join correctly without panicking again.
    /// # Ok(()) }
    /// ```
    pub async fn submit_async<F>(&self, mut future: F) -> F::Output
    where
        F: Send + Future,
        F::Output: Send,
    {
        // Parker to use during polling.
        let parker = Parker::new();
        // Stack location where the output of the compuation is stored.
        let mut output = None;

        unsafe {
            let wait_future = WaitFuture {
                future: ptr::NonNull::from(&mut future),
                output: ptr::NonNull::from(&mut output),
                parker: ptr::NonNull::from(&parker),
                complete: false,
                shared: self.shared.as_ref(),
            };

            wait_future.await
        }
    }

    /// Move the provided `value` onto the background thread and drop it.
    ///
    /// This is necessary for values which uses [Tag] to ensure that a type is
    /// not dropped incorrectly.
    ///
    /// # Examples
    ///
    /// ```rust
    /// struct Foo(ste::Tag);
    ///
    /// impl Drop for Foo {
    ///     fn drop(&mut self) {
    ///         self.0.ensure_on_thread();
    ///     }
    /// }
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let thread = ste::spawn();
    ///
    /// let foo = thread.submit(|| Foo(ste::Tag::current_thread()));
    /// thread.drop(foo);
    ///
    /// thread.join();
    /// # Ok(()) }
    /// ```
    pub fn drop<T>(&self, value: T)
    where
        T: Send,
    {
        self.submit(move || drop(value));
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
    /// let thread = ste::spawn();
    ///
    /// let mut n = 10;
    /// thread.submit(|| n += 10);
    /// assert_eq!(20, n);
    ///
    /// thread.join();
    /// # Ok(()) }    
    /// ```
    pub fn join(mut self) {
        if let Some(handle) = self.handle.take() {
            unsafe { self.shared.as_ref().outer_join() };

            if handle.join().is_err() {
                panic!("background thread panicked");
            }
        }
    }

    /// Construct the tag that is associated with the current thread externally
    /// from the thread.
    ///
    /// # Examples
    ///
    /// ```rust
    /// struct Foo(ste::Tag);
    ///
    /// impl Foo {
    ///     fn say_hello(&self) {
    ///         self.0.ensure_on_thread();
    ///         println!("Hello World");
    ///     }
    /// }
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let thread = ste::spawn();
    ///
    /// let foo = Foo(thread.tag());
    ///
    /// thread.submit(|| {
    ///     foo.say_hello();
    /// });
    ///
    /// thread.join();
    /// # Ok(()) }
    /// ```
    pub fn tag(&self) -> Tag {
        Tag(self.shared.as_ptr() as usize)
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        // Note: we can safely ignore the result, because it will only error in
        // case the background thread has panicked. At which point we're still
        // free to assume it's no longer using the shared state.
        if let Some(handle) = self.handle.take() {
            unsafe { self.shared.as_ref().outer_join() };
            let _ = handle.join();
        }

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
    #[cfg(feature = "tokio")]
    tokio: Option<tokio::runtime::Handle>,
}

impl Builder {
    /// Construct a new builder.
    pub fn new() -> Self {
        Self {
            prelude: None,
            #[cfg(feature = "tokio")]
            tokio: None,
        }
    }

    /// Enable tokio support.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> anyhow::Result<()> {
    /// let thread = ste::Builder::new().with_tokio().build()?;
    ///
    /// thread.submit_async(async {
    ///     tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    ///     println!("Hello World!");
    /// });
    ///
    /// thread.join();
    /// # Ok(()) }
    /// ```
    #[cfg(feature = "tokio")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
    pub fn with_tokio(self) -> Self {
        Self {
            tokio: Some(::tokio::runtime::Handle::current()),
            ..self
        }
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
            ..self
        }
    }

    /// Construct the background thread.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> anyhow::Result<()> {
    /// let thread = ste::Builder::new().build()?;
    /// thread.join();
    /// # Ok(()) }
    /// ```
    pub fn build(self) -> io::Result<Thread> {
        let shared = ptr::NonNull::from(Box::leak(Box::new(Shared::new())));

        let prelude = self.prelude;
        #[cfg(feature = "tokio")]
        let tokio = self.tokio;

        let shared2 = RawSend(shared);

        let handle = thread::Builder::new()
            .name(String::from("ste-thread"))
            .spawn(move || {
                let RawSend(shared) = shared2;

                #[cfg(feature = "tokio")]
                let _guard = tokio.as_ref().map(|h| h.enter());

                worker::run(prelude, shared)
            })?;

        Ok(Thread {
            shared,
            handle: Some(handle),
        })
    }
}
