pub(crate) use ::std::sync::atomic;
pub(crate) use ::std::sync::Arc;
use std::ops::{Deref, DerefMut};

pub(crate) struct MutexGuard<'a, T> {
    inner: ::std::sync::MutexGuard<'a, T>,
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.inner.deref()
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.inner.deref_mut()
    }
}

pub(crate) struct Mutex<T> {
    inner: ::std::sync::Mutex<T>,
}

impl<T> Mutex<T> {
    pub(crate) fn new(value: T) -> Self {
        Self {
            inner: ::std::sync::Mutex::new(value),
        }
    }

    pub(crate) fn lock(&self) -> MutexGuard<'_, T> {
        MutexGuard {
            inner: self.inner.lock().unwrap(),
        }
    }
}
