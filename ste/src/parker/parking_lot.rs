use parking_lot_core::{DEFAULT_PARK_TOKEN, DEFAULT_UNPARK_TOKEN};

#[repr(transparent)]
pub struct Parker {
    key: usize,
}

#[repr(transparent)]
pub struct Unparker {
    key: usize,
}

/// Construct a new parker.
pub(crate) fn new<P>(ptr: *mut P) -> (Parker, Unparker) {
    let key = ptr as usize;
    (Parker { key }, Unparker { key })
}

impl Parker {
    pub(crate) fn park<F>(&self, _: F) {
        unsafe {
            parking_lot_core::park(
                self.key,
                || true,
                || {},
                |_, _| {},
                DEFAULT_PARK_TOKEN,
                None,
            );
        }
    }
}

impl Unparker {
    pub(crate) fn unpark_one(&self) -> bool {
        // Safety: the storage address is shared by the entity submitting the
        // task.
        unsafe {
            parking_lot_core::unpark_one(self.key, |_| DEFAULT_UNPARK_TOKEN).unparked_threads == 1
        }
    }
}
