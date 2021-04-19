use crate::libc as c;
use crate::pulse::{error, ContextState, Error, Result};
use pulse_sys as pulse;
use std::ptr;

/// Structure holding onto a registered callback.
pub struct Callback {
    data: *mut (),
    drop: unsafe fn(*mut ()),
}

impl Drop for Callback {
    fn drop(&mut self) {
        unsafe {
            (self.drop)(self.data);
        }
    }
}

/// An opaque connection context to a daemon.
///
/// See [MainLoop::context][super::MainLoop::context].
pub struct Context {
    pub(super) handle: ptr::NonNull<pulse::pa_context>,
    pub(super) callbacks: Vec<Callback>,
}

impl Context {
    /// Set a callback function that is called whenever the context status
    /// changes.
    pub fn set_callback<C>(&mut self, cb: C) -> Result<()>
    where
        C: 'static + FnMut(&mut Context) -> Result<()>,
    {
        let cx = self as *mut _;

        let cb = Box::into_raw(Box::new(Wrapper { cx, cb }));

        self.callbacks.push(Callback {
            data: cb as *mut (),
            drop: drop_impl::<Wrapper<C>>,
        });

        unsafe {
            return ffi_error!(pulse::pa_context_set_state_callback(
                self.handle.as_mut(),
                Some(callback::<C>),
                cb as *mut c::c_void
            ));
        }

        extern "C" fn callback<C>(cx: *mut pulse::pa_context, cb: *mut c::c_void)
        where
            C: 'static + FnMut(&mut Context) -> Result<()>,
        {
            unsafe {
                let cb = &mut *(cb as *mut Wrapper<C>);
                debug_assert!(ptr::eq(cx, (*cb.cx).handle.as_ptr()));
                cb.call();
            }
        }

        // Wire up the type `C` to be dropped once the context is dropped.
        unsafe fn drop_impl<C>(data: *mut ()) {
            ptr::drop_in_place(data as *mut C);
        }

        struct Wrapper<C> {
            cx: *mut Context,
            cb: C,
        }

        impl<C> Wrapper<C>
        where
            C: 'static + FnMut(&mut Context) -> Result<()>,
        {
            unsafe fn call(&mut self) {
                let cx = &mut (*self.cx);
                error::capture(|| (self.cb)(cx));
            }
        }
    }

    /// Connect the context to the specified server.
    pub fn connect(&mut self) -> Result<()> {
        unsafe {
            error!(
                self,
                pulse::pa_context_connect(self.handle.as_mut(), ptr::null(), 0, ptr::null())
            )?;
            Ok(())
        }
    }

    /// Get the current state of the context.
    pub fn state(&self) -> Result<ContextState> {
        unsafe {
            let state = pulse::pa_context_get_state(self.handle.as_ptr());
            ContextState::from_value(state).ok_or_else(|| Error::BadContextState(state))
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            pulse::pa_context_unref(self.handle.as_mut());
        }
    }
}
