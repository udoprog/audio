use crate::driver::atomic_waker::AtomicWaker;
use crate::libc as c;
use crate::loom::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use crate::loom::sync::{Arc, Mutex};
use crate::loom::thread;
use crate::unix::errno::Errno;
use std::collections::HashMap;
use std::mem;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("system error: {0}")]
    Sys(#[from] Errno),
}

pub type Result<T, E = Error> = ::std::result::Result<T, E>;

/// The token associated with the current waiter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Token(c::c_int);

/// A guard for the returned events of the poll handler.
///
/// Dropping this handle will allow the background thread to poll the handler
/// again.
///
/// Constructed by waiting on [PollHandle::returned_events].
pub struct PollEventsGuard<'a> {
    events: c::c_short,
    shared: &'a Shared,
    token: Token,
}

impl PollEventsGuard<'_> {
    /// Access the returned events.
    pub fn events(&self) -> c::c_short {
        self.events
    }
}

impl Drop for PollEventsGuard<'_> {
    fn drop(&mut self) {
        self.shared.holders.lock().released.push(self.token);

        if let Err(e) = self.shared.parker.send(1) {
            log::error!("failed to unpark background thread: {}", e);
        }
    }
}

/// A handle to a registered poll file descriptor.
///
/// Constructed with [Handle::register].
pub struct PollHandle {
    shared: Arc<Shared>,
    waker: Arc<Waker>,
}

impl PollHandle {
    /// Wait for events to be triggered on the background driver and return a
    /// guard to the events.
    ///
    /// Once this guard is dropped the driver will be released to register more
    /// interest.
    pub async fn returned_events(&self) -> PollEventsGuard<'_> {
        use std::future::Future;
        use std::pin::Pin;
        use std::task::{Context, Poll};

        return ReturnedEvents(self).await;

        struct ReturnedEvents<'a>(&'a PollHandle);

        impl<'a> Future for ReturnedEvents<'a> {
            type Output = PollEventsGuard<'a>;

            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                self.0.waker.waker.register_by_ref(cx.waker());
                let returned_events = self.0.waker.returned_events.swap(0, Ordering::Acquire);

                if returned_events != 0 {
                    Poll::Ready(PollEventsGuard {
                        events: returned_events as c::c_short,
                        shared: &*self.0.shared,
                        token: self.0.waker.token(),
                    })
                } else {
                    Poll::Pending
                }
            }
        }
    }
}

impl Drop for PollHandle {
    fn drop(&mut self) {
        self.shared.holders.lock().removed.push(self.waker.token());

        if let Err(e) = self.shared.parker.send(1) {
            log::error!("failed to unpark background thread: {}", e);
        }
    }
}

/// Data on the waker for a handle.
pub(crate) struct Waker {
    /// The waker to call when waking up the task waiting for events.
    pub(crate) waker: AtomicWaker,
    /// The descriptors associated with this waker.
    descriptor: c::pollfd,
    /// The last revents decoded. `None` if no events are ready.
    returned_events: AtomicUsize,
}

impl Waker {
    /// Get the token associated with this waker.
    ///
    /// Note: always the first file descriptor.
    fn token(&self) -> Token {
        Token(self.descriptor.fd)
    }
}

pub(crate) struct Shared {
    pub(crate) running: AtomicBool,
    pub(crate) holders: Mutex<Events>,
    pub(crate) parker: EventFd,
}

#[derive(Default)]
pub(crate) struct Events {
    pub(crate) added: Vec<Arc<Waker>>,
    pub(crate) released: Vec<Token>,
    pub(crate) removed: Vec<Token>,
}

impl Events {
    // Process all queued elements in the driver.
    fn process(&mut self, driver: &mut Driver, wakers: &mut Vec<Arc<Waker>>) -> Result<()> {
        let mut added = mem::replace(&mut self.added, Vec::new());

        for waker in added.drain(..) {
            let loc = Loc {
                descriptor: driver.descriptors.len(),
                waker: wakers.len(),
            };

            driver.locations.insert(waker.token(), loc);
            driver.descriptors.push(waker.descriptor);
            wakers.push(waker);
        }

        let mut removed = mem::replace(&mut self.removed, Vec::new());

        for token in removed.drain(..) {
            if let Some(loc) = driver.locations.remove(&token) {
                driver.descriptors.swap_remove(loc.descriptor);
                wakers.swap_remove(loc.waker);
                // NB: redirect swap removed.
                let token = wakers[loc.waker].token();
                driver.locations.insert(token, loc);
            }
        }

        let mut released = mem::replace(&mut self.released, Vec::new());

        for r in released.drain(..) {
            if let Some(Loc { descriptor, waker }) = driver.locations.get(&r) {
                driver.descriptors[*descriptor].fd = wakers[*waker].descriptor.fd;
            }
        }

        self.added = added;
        self.removed = removed;
        self.released = released;
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
            holders: Mutex::new(Events::default()),
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

    /// Register a pollfd for interest in events.
    ///
    /// Dropping the returned [PollHandle] will unregister interest.
    pub fn register(&self, descriptor: c::pollfd) -> Result<PollHandle, Errno> {
        let waker = Arc::new(Waker {
            waker: AtomicWaker::new(),
            descriptor,
            returned_events: AtomicUsize::new(0),
        });

        self.shared.holders.lock().added.push(waker.clone());

        self.shared.parker.send(1)?;

        Ok(PollHandle {
            shared: self.shared.clone(),
            waker,
        })
    }

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

#[derive(Debug, Clone, Copy)]
struct Loc {
    descriptor: usize,
    waker: usize,
}

struct Driver {
    /// Location of a given token.
    locations: HashMap<Token, Loc>,
    /// The descriptors being driven.
    descriptors: Vec<libc::pollfd>,
}

impl Driver {
    fn run(mut self, guard: &mut PanicGuard) -> Result<()> {
        while guard.shared.running.load(Ordering::Acquire) {
            let mut result = unsafe {
                errno!(libc::poll(
                    self.descriptors.as_mut_ptr(),
                    self.descriptors.len() as libc::c_ulong,
                    -1,
                ))?
            };

            let mut notified = false;

            for (n, e) in self.descriptors.iter_mut().enumerate() {
                if e.revents == 0 {
                    continue;
                }

                if result == 0 {
                    break;
                }

                result -= 1;

                if n == 0 {
                    let _ = guard.shared.parker.recv()?;
                    notified = true;
                    continue;
                }

                // Disable file descriptor and wakeup the task to receive the
                // returned events.
                e.fd = -1;
                let waker = &guard.wakers[n - 1];
                waker
                    .returned_events
                    .store(std::mem::take(&mut e.revents) as usize, Ordering::Release);
                waker.waker.wake();
            }

            if notified {
                let mut holders = guard.shared.holders.lock();
                holders.process(&mut self, &mut guard.wakers)?;
            }
        }

        return Ok(());
    }

    fn start(shared: Arc<Shared>) {
        let state = Driver {
            locations: HashMap::new(),
            descriptors: vec![libc::pollfd {
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

/// Helper wrapper around an eventfd.
pub(crate) struct EventFd {
    fd: c::c_int,
}

impl EventFd {
    fn new() -> Result<Self> {
        unsafe {
            Ok(Self {
                fd: errno!(c::eventfd(0, c::EFD_NONBLOCK))?,
            })
        }
    }

    /// Add the given number to the eventfd.
    fn send(&self, v: u64) -> Result<(), Errno> {
        unsafe {
            let n = v.to_ne_bytes();
            errno!(c::write(self.fd, n.as_ptr() as *const c::c_void, 8))?;
            Ok(())
        }
    }

    /// Read the next value from the eventfd.
    fn recv(&self) -> Result<u64> {
        unsafe {
            let mut bytes = [0u8; 8];
            let read = errno!(c::read(self.fd, bytes.as_mut_ptr() as *mut c::c_void, 8))?;

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
