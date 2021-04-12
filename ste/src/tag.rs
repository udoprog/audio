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
/// If that is done, you can safely implement [Send] for the type.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Tag(pub(super) usize);

impl Tag {
    /// Get the tag associated with the current thread.
    ///
    /// # Panics
    ///
    /// Panics if not running on a tagged thread.
    pub fn current_thread() -> Self {
        match THREAD_TAG.with(|tag| tag.get()) {
            Tag(0) => panic!("not running on a tagged thread"),
            tag => tag,
        }
    }

    /// Ensure that the tag is currently executing on the thread that created
    /// it.
    ///
    /// # Panics
    ///
    /// Panics unless called on the same thread that the tag was created on.
    pub fn ensure_on_thread(&self) {
        let current = THREAD_TAG.with(|tag| tag.get());

        if *self != current {
            panic!(
                "cannot operate on tagged element unless on the correct thread, \
                got {:?} but expected {:?}",
                current, self
            );
        }
    }
}

impl fmt::Debug for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Tag")
            .field(&format_args!("{:#x}", self.0))
            .finish()
    }
}
