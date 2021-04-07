use std::thread;

pub struct Parker;

#[repr(transparent)]
pub struct Unparker {
    thread: thread::Thread,
}

/// Construct a new parker.
pub(crate) fn new<P>(_: *mut P) -> (Parker, Unparker) {
    (
        Parker,
        Unparker {
            thread: thread::current(),
        },
    )
}

impl Parker {
    pub(crate) fn park<T>(&self, is_set: T)
    where
        T: Fn() -> bool,
    {
        loop {
            thread::park();

            if is_set() {
                break;
            }
        }
    }
}

impl Unparker {
    pub(crate) fn unpark_one(&self) -> bool {
        self.thread.unpark();
        true
    }
}
