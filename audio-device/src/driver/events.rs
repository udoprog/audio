//! A Windows event executor.

use crate::driver::atomic_waker::AtomicWaker;
use crate::loom::sync::atomic::{AtomicBool, Ordering};
use crate::loom::sync::{Arc, Mutex};
use crate::loom::thread;
use crate::windows::{AsyncEvent, Event, RawEvent as _};
use std::io;
use std::mem;
use thiserror::Error;
use windows_sys::Windows::Win32::SystemServices as ss;
use windows_sys::Windows::Win32::WindowsProgramming as wp;

#[derive(Debug, Error)]
#[error("background thread panicked")]
pub struct Panicked(());

/// Data on the waker for a handle.
pub(crate) struct Waker {
    pub(crate) ready: AtomicBool,
    pub(crate) waker: AtomicWaker,
    handle: ss::HANDLE,
}

pub(crate) struct Shared {
    pub(crate) running: AtomicBool,
    pub(crate) holders: Mutex<Holders>,
    pub(crate) parker: Event,
}

#[derive(Default)]
pub(crate) struct Holders {
    pub(crate) added: Vec<Arc<Waker>>,
    pub(crate) removed: Vec<Event>,
}

pub struct Handle {
    thread: Option<thread::JoinHandle<()>>,
    shared: Arc<Shared>,
}

impl Handle {
    /// Construct a new events windows driver and return its handle.
    pub fn new() -> windows::Result<Self> {
        let shared = Arc::new(Shared {
            running: AtomicBool::new(true),
            holders: Mutex::new(Holders::default()),
            parker: Event::new(false, false)?,
        });

        let thread = thread::spawn({
            let shared = shared.clone();
            || Driver::start(shared)
        });

        let handle = Self {
            thread: Some(thread),
            shared,
        };

        Ok(handle)
    }

    /// Construct an asynchronous event associated with the current handle.
    pub fn event(&self, initial_state: bool) -> windows::Result<AsyncEvent> {
        let event = Event::new(false, initial_state)?;
        let handle = unsafe { event.raw_event() };

        let waker = Arc::new(Waker {
            ready: AtomicBool::new(false),
            waker: AtomicWaker::new(),
            handle,
        });

        self.shared
            .holders
            .lock()
            .unwrap()
            .added
            .push(waker.clone());
        self.shared.parker.set();

        Ok(AsyncEvent::new(self.shared.clone(), waker, event))
    }

    /// Join the current handle.
    pub fn join(mut self) -> Result<(), Panicked> {
        self.inner_join()?;
        Ok(())
    }

    fn inner_join(&mut self) -> Result<(), Panicked> {
        if let Some(thread) = self.thread.take() {
            self.shared.running.store(false, Ordering::Release);
            self.shared.parker.set();
            return thread.join().map_err(|_| Panicked(()));
        }

        Ok(())
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        let _ = self.inner_join();
    }
}

struct Driver {
    events: Vec<ss::HANDLE>,
    wakers: Vec<Arc<Waker>>,
    shared: Arc<Shared>,
}

impl Driver {
    fn run(mut self) {
        let guard = PanicGuard {
            shared: &*self.shared,
            wakers: &mut self.wakers,
        };

        while self.shared.running.load(Ordering::Acquire) {
            let result = unsafe {
                ss::WaitForMultipleObjects(
                    self.events.len() as u32,
                    self.events.as_ptr(),
                    ss::FALSE,
                    wp::INFINITE,
                )
            };

            match result {
                ss::WAIT_RETURN_CAUSE::WAIT_ABANDONED_0 => panic!("wait abandoned"),
                ss::WAIT_RETURN_CAUSE::WAIT_TIMEOUT => panic!("timed out"),
                ss::WAIT_RETURN_CAUSE::WAIT_FAILED => {
                    panic!("wait failed: {}", io::Error::last_os_error())
                }
                other => {
                    let base = ss::WAIT_RETURN_CAUSE::WAIT_OBJECT_0.0;
                    let other = other.0;

                    if other < base {
                        panic!("other out of bounds; other = {}", other);
                    }

                    let index = (other - base) as usize;

                    if !(index < self.events.len()) {
                        panic!("wakeup out of bounds; index = {}", index);
                    }

                    // NB: index 0 is the wakeup to notify once things are
                    // added, any other is a legit registered event.
                    if index > 0 {
                        if let Some(waker) = guard.wakers.get(index - 1) {
                            waker.ready.store(true, Ordering::Release);
                            waker.waker.wake();
                        }

                        continue;
                    }
                }
            }

            let mut holders = self.shared.holders.lock().unwrap();
            let mut added = mem::replace(&mut holders.added, Vec::new());

            for waker in added.drain(..) {
                self.events.push(waker.handle);
                guard.wakers.push(waker);
            }

            holders.added = added;

            let mut removed = mem::replace(&mut holders.removed, Vec::new());

            for event in removed.drain(..) {
                let removed = unsafe { event.raw_event().0 };

                if let Some(index) = guard.wakers.iter().position(|w| w.handle.0 == removed) {
                    guard.wakers.swap_remove(index);
                    self.events.swap_remove(index + 1);
                }
            }

            holders.removed = removed;
        }

        mem::forget(guard);

        /// Wrap a panic guard around self which will release any resources it
        /// has allocated when dropped and mark itself as panicked.
        struct PanicGuard<'a> {
            shared: &'a Shared,
            wakers: &'a mut Vec<Arc<Waker>>,
        }

        impl Drop for PanicGuard<'_> {
            fn drop(&mut self) {
                self.shared.running.store(false, Ordering::Release);

                // Wake up every waker so that they can observe the panic.
                for waker in self.wakers.iter() {
                    waker.waker.wake();
                }
            }
        }
    }

    fn start(shared: Arc<Shared>) {
        let state = Driver {
            events: vec![unsafe { shared.parker.raw_event() }],
            wakers: vec![],
            shared,
        };

        state.run()
    }
}
