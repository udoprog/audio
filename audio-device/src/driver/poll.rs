use crate::driver::atomic_waker::AtomicWaker;
use crate::loom::sync::atomic::{AtomicBool, Ordering};
use crate::loom::sync::{Arc, Mutex};
use crate::loom::thread;
use crate::unix::errno::Errno;
use std::mem;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("system error: {0}")]
    Sys(#[from] Errno),
}

pub type Result<T, E = Error> = ::std::result::Result<T, E>;

pub(crate) struct EventFd {
    fd: libc::c_int,
}

impl EventFd {
    fn new() -> Result<Self> {
        unsafe {
            Ok(Self {
                fd: errno!(libc::eventfd(0, libc::EFD_NONBLOCK))?,
            })
        }
    }

    /// Add the given number to the eventfd.
    fn send(&self, v: u64) -> Result<()> {
        unsafe {
            let n = v.to_ne_bytes();
            errno!(libc::write(self.fd, n.as_ptr() as *const libc::c_void, 8))?;
            Ok(())
        }
    }

    /// Read the next value from the eventfd.
    fn recv(&self) -> Result<u64> {
        unsafe {
            let mut bytes = [0u8; 8];
            let read = errno!(libc::read(
                self.fd,
                bytes.as_mut_ptr() as *mut libc::c_void,
                8
            ))?;

            assert!(read == 8);
            Ok(u64::from_ne_bytes(bytes))
        }
    }
}

impl Drop for EventFd {
    fn drop(&mut self) {
        unsafe {
            let _ = libc::close(self.fd);
        }
    }
}

/// Data on the waker for a handle.
pub(crate) struct Waker {
    pub(crate) waker: Arc<AtomicWaker>,
    fd: libc::pollfd,
}

pub(crate) struct Shared {
    pub(crate) running: AtomicBool,
    pub(crate) holders: Mutex<Holders>,
    pub(crate) parker: EventFd,
}

#[derive(Default)]
pub(crate) struct Holders {
    pub(crate) added: Vec<Arc<Waker>>,
    pub(crate) removed: Vec<libc::pollfd>,
}

impl Holders {
    // Process all queued elements in the driver.
    fn process(&mut self, driver: &mut Driver, wakers: &mut Vec<Arc<Waker>>) -> Result<()> {
        let mut added = mem::replace(&mut self.added, Vec::new());

        for waker in added.drain(..) {
            driver.files.push(waker.fd);
            wakers.push(waker);
        }

        let mut removed = mem::replace(&mut self.removed, Vec::new());

        for pollfd in removed.drain(..) {
            if let Some(index) = wakers.iter().position(|w| w.fd.fd == pollfd.fd) {
                driver.files.swap_remove(index + 1);
                wakers.swap_remove(index);
            }

            unsafe {
                errno!(libc::close(pollfd.fd))?;
            }
        }

        self.added = added;
        self.removed = removed;
        Ok(())
    }
}

cfg_poll_driver! {
    /// An executor to drive things which are woken up by polling.
    pub struct Poll {
        thread: Option<thread::JoinHandle<()>>,
        shared: Arc<Shared>,
    }
}

impl Poll {
    /// Construct a new events windows event object driver and return its
    /// handle.
    pub fn new() -> Result<Self> {
        let shared = Arc::new(Shared {
            running: AtomicBool::new(true),
            holders: Mutex::new(Holders::default()),
            parker: EventFd::new()?,
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

    /// Test out.
    pub fn test(&self) -> Result<()> {
        self.shared.parker.send(1)?;
        Ok(())
    }

    /// Construct an asynchronous event associated with the current handle.
    /*pub fn event(&self, initial_state: bool) -> Result<AsyncEvent> {
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
    }*/

    /// Join the current handle.
    ///
    /// # Panics
    ///
    /// This panics if the background thread panicked. But this should only ever
    /// happen if there's a bug.
    pub fn join(mut self) {
        self.inner_join();
    }

    fn inner_join(&mut self) {
        if let Some(thread) = self.thread.take() {
            self.shared.running.store(false, Ordering::Release);

            if let Err(errno) = self.shared.parker.send(0) {
                panic!("failed to set event: {}", errno);
            }

            if thread.join().is_err() {
                panic!("event handler thread panicked");
            }
        }
    }
}

impl Drop for Poll {
    fn drop(&mut self) {
        let _ = self.inner_join();
    }
}

struct Driver {
    files: Vec<libc::pollfd>,
}

impl Driver {
    fn run(mut self, guard: &mut PanicGuard) -> Result<()> {
        while guard.shared.running.load(Ordering::Acquire) {
            let result = unsafe {
                libc::poll(
                    self.files.as_mut_ptr(),
                    self.files.len() as libc::c_ulong,
                    -1,
                )
            };

            if result == -1 {
                panic!("poll: {}", Errno::from_i32(-result));
            }

            let mut notified = false;

            for (n, e) in self.files.iter_mut().enumerate() {
                if e.revents == 0 {
                    continue;
                }

                if n == 0 {
                    let _ = guard.shared.parker.recv()?;
                    notified = true;
                    continue;
                }

                panic!("don't know how to handle legit events yet!")
            }

            if notified {
                guard
                    .shared
                    .holders
                    .lock()
                    .unwrap()
                    .process(&mut self, &mut guard.wakers)?;
            }
        }

        return Ok(());
    }

    fn start(shared: Arc<Shared>) {
        let state = Driver {
            files: vec![libc::pollfd {
                fd: shared.parker.fd,
                events: libc::POLLIN,
                revents: 0,
            }],
        };

        let mut guard = PanicGuard {
            shared,
            wakers: vec![],
        };

        if let Err(e) = state.run(&mut guard) {
            panic!("poll thread errored: {}", e)
        }

        mem::forget(guard);
    }
}

/// Wrap a panic guard around self which will release any resources it
/// has allocated when dropped and mark itself as panicked.
struct PanicGuard {
    shared: Arc<Shared>,
    wakers: Vec<Arc<Waker>>,
}

impl Drop for PanicGuard {
    fn drop(&mut self) {
        self.shared.running.store(false, Ordering::Release);

        // Wake up every waker so that they can observe the panic.
        for waker in self.wakers.iter() {
            waker.waker.wake();
        }
    }
}
