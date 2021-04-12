use std::cell::Cell;
use std::fmt;

thread_local! {
    static THREAD_TAG: Cell<Tag> = Cell::new(Tag(0));
}

/// Run the given closure with the specified tag.
pub(super) fn with_tag<F, T>(tag: Tag, f: F) -> T
where
    F: FnOnce() -> T,
{
    return THREAD_TAG.with(|w| {
        let _guard = Guard(w.replace(tag));
        f()
    });

    struct Guard(Tag);

    impl Drop for Guard {
        fn drop(&mut self) {
            THREAD_TAG.with(|w| {
                w.set(self.0);
            });
        }
    }
}

/// A tag associated with a thread. Threads which are executed with
/// [Thread][super::Thread] support tagging.
///
/// You must ensure that any thread trying to use values first is checked with
/// the current tag through [Tag::ensure_on_thread]. This includes everything
/// which poses a potential thread safety risk.
///
/// This includes, but is not limited to:
/// * Accessing or mutating any racy data or APIs, such as `Cell<T>`.
/// * The type that is tagged, (and any nested types) [drop][Drop::drop]
///   implementation.
///
/// If all of the above is satifised, you can safely implement [Send] and [Sync]
/// for the type. Make sure to include a comprehensive safety message such as:
///
/// ```rust
/// # struct Foo;
/// // Safety: the structure is explicitly tagged with the thread that created
/// // it, and we ensure everywhere (including drop implementations) where racy
/// // access might occur that it is on the thread that created it.
/// unsafe impl Send for Foo {}
/// ```
///
/// Tags can only be correctly constructed in two ways:
/// * By calling [Tag::current_thread] if inside of a thread context. Such as
///   [Thread::submit][super::Thread::submit] or
///   [Thread::submit_async][super::Thread::submit_async].
/// * Externally by calling [Thread::tag][super::Thread::tag].
///
/// # Examples
///
/// ```rust
/// use std::cell::Cell;
///
/// struct Foo {
///     tag: ste::Tag,
///     data: Cell<usize>,
/// }
///
/// impl Foo {
///     fn new() -> Self {
///         Self {
///             tag: ste::Tag::current_thread(),
///             data: Cell::new(42),
///         }
///     }
///
///     fn say_hello(&self) {
///         self.tag.ensure_on_thread();
///         println!("Hello from Foo: {}", self.data.get());
///     }
/// }
///
/// // Safety: the structure is explicitly tagged with the thread that created
/// // it, and we ensure everywhere (including drop implementations) where racy
/// // access might occur that it is on the thread that created it.
/// unsafe impl Send for Foo {}
/// unsafe impl Sync for Foo {}
///
/// # fn main() -> anyhow::Result<()> {
/// let thread = ste::Thread::new()?;
///
/// let foo = thread.submit(|| Foo::new())?;
///
/// assert!(!foo.tag.is_on_thread());
///
/// thread.submit(|| foo.say_hello());
///
/// thread.join()?;
/// # Ok(()) }
/// ```
///
/// Incorrect use of the tagged struct **must** panic:
///
/// ```rust,should_panic
/// # struct Foo { tag: ste::Tag }
/// # impl Foo {
/// #     fn new() -> Self { Self { tag: ste::Tag::current_thread() } }
/// #     fn say_hello(&self) { self.tag.ensure_on_thread(); }
/// # }
/// # fn main() -> anyhow::Result<()> {
/// let thread = ste::Thread::new()?;
///
/// let foo = thread.submit(|| Foo::new())?;
///
/// assert!(!foo.tag.is_on_thread());
///
/// foo.say_hello(); // <- oops, this panics!
///
/// thread.join()?;
/// # Ok(()) }
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Tag(pub(super) usize);

impl Tag {
    /// Get the tag associated with the current thread.
    ///
    /// See [Tag] documentation for how to use correctly.
    ///
    /// # Panics
    ///
    /// Panics if not running on a tagged thread. Tagged threads are the ones
    /// created with [Thread][super::Thread].
    pub fn current_thread() -> Self {
        match THREAD_TAG.with(|tag| tag.get()) {
            Tag(0) => panic!("not running on a tagged thread"),
            tag => tag,
        }
    }

    /// Ensure that the tag is currently executing on the thread that created
    /// it.
    ///
    /// See [Tag] documentation for how to use.
    ///
    /// # Panics
    ///
    /// Panics if not running on a tagged thread. Tagged threads are the ones
    /// created with [Thread][super::Thread].
    ///
    /// Also panics unless called on the same thread that the tag was created
    /// on.
    pub fn ensure_on_thread(&self) {
        let current = Self::current_thread();

        if *self != current {
            panic!(
                "cannot operate on tagged element unless on the correct thread, \
                got {:?} but expected {:?}",
                current, self
            );
        }
    }

    /// Test if we're currently on the tagged thread.
    ///
    /// See [Tag] documentation for how to use.
    pub fn is_on_thread(&self) -> bool {
        THREAD_TAG.with(|tag| match tag.get() {
            Tag(0) => false,
            tag => *self == tag,
        })
    }
}

impl fmt::Debug for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Tag")
            .field(&format_args!("{:#x}", self.0))
            .finish()
    }
}
