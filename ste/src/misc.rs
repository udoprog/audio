use std::ptr::NonNull;

/// Small helper for sending pointers which are not send.
#[repr(transparent)]
pub(crate) struct RawSend<T>
where
    T: ?Sized,
{
    ptr: NonNull<T>,
}

impl<T> RawSend<T>
where
    T: ?Sized,
{
    /// Construct a new raw send wrapper.
    pub(crate) const fn new(ptr: NonNull<T>) -> Self {
        RawSend { ptr }
    }

    /// Get the inner pointer.
    pub(crate) const fn into_inner(self) -> NonNull<T> {
        self.ptr
    }

    /// Get a reference to the inner value.
    pub(crate) fn as_ref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }

    /// Get a mutable reference to the inner value.
    pub(crate) unsafe fn as_mut(&mut self) -> &mut T {
        unsafe { self.ptr.as_mut() }
    }
}

// Safety: this is limited to the module and guaranteed to be correct.
unsafe impl<T> Send for RawSend<T> where T: ?Sized {}
