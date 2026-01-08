use core::ops::{Deref, DerefMut};
pub(crate) use core::sync::atomic;

pub(crate) use alloc::sync::Arc;

use std::sync::{Mutex as StdMutex, MutexGuard as StdMutexGuard};

pub(crate) struct MutexGuard<'a, T> {
    inner: StdMutexGuard<'a, T>,
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
    inner: StdMutex<T>,
}

impl<T> Mutex<T> {
    pub(crate) fn new(value: T) -> Self {
        Self {
            inner: StdMutex::new(value),
        }
    }

    pub(crate) fn lock(&self) -> MutexGuard<'_, T> {
        MutexGuard {
            inner: self.inner.lock().unwrap(),
        }
    }
}
