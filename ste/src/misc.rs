use std::ptr;

/// Small helper for sending pointers which are not send.
#[repr(transparent)]
pub(crate) struct RawSend<T>(pub(crate) ptr::NonNull<T>)
where
    T: ?Sized;

// Safety: this is limited to the module and guaranteed to be correct.
unsafe impl<T> Send for RawSend<T> where T: ?Sized {}
