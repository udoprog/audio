use std::cell::Cell;
use std::fmt;
use std::mem;
use std::ops;
use std::ptr;

thread_local! {
    static THREAD_TAG: Cell<Tag> = Cell::new(Tag(0));
}

/// An object `T` which can only be used on the thread with the corresponding
/// tag.
///
/// # Dropping tagged elements
///
/// Dropping a tagged container *outside* of its thread and the underlying `T`
/// implements [Drop] will cause it to panic.
///
/// The correct way to drop it is to move it back onto the thread, use use the
/// provided [Thread::drop][super::Thread::drop].
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
pub struct Tagged<T> {
    tag: Tag,
    target: mem::ManuallyDrop<T>,
}

impl<T> Tagged<T> {
    /// Construct a new container tagged with the current thread.
    pub fn new(target: T) -> Self {
        Self {
            tag: tag(),
            target: mem::ManuallyDrop::new(target),
        }
    }

    fn ensure_on_thread(&self) {
        let current = THREAD_TAG.with(|tag| tag.get());

        if self.tag != current {
            panic!(
                "cannot operate on tagged element unless on the correct thread, \
                got {:?} but expected {:?}",
                current, self.tag
            );
        }
    }
}

// Safety: Tagged can be marked by sync, because it effectively prevents the
// underlying object `T` from being used in any way except on the thread which
// created it.
unsafe impl<T> Send for Tagged<T> {}
unsafe impl<T> Sync for Tagged<T> {}

impl<T> Drop for Tagged<T> {
    fn drop(&mut self) {
        if !mem::needs_drop::<T>() {
            return;
        }

        self.ensure_on_thread();

        // Safety: We own the target value, and we are the only ones
        // controlling whether it is dropped or not.
        unsafe {
            ptr::drop_in_place(&mut *self.target);
        }
    }
}

impl<T> ops::Deref for Tagged<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.ensure_on_thread();
        &self.target
    }
}

impl<T> ops::DerefMut for Tagged<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ensure_on_thread();
        &mut self.target
    }
}

/// Get the current tag.
///
/// # Panics
///
/// Panics if not running on a tagged thread.
fn tag() -> Tag {
    match THREAD_TAG.with(|tag| tag.get()) {
        Tag(0) => panic!("not running on a tagged thread"),
        tag => tag,
    }
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

/// A tag associated with a thread.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub(super) struct Tag(pub(super) usize);

impl fmt::Debug for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Tag")
            .field(&format_args!("{:#x}", self.0))
            .finish()
    }
}
