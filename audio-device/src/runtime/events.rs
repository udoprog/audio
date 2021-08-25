use crate::loom::sync::atomic::{AtomicBool, Ordering};
use crate::loom::sync::{Arc, Mutex};
use crate::loom::thread;
use crate::runtime::atomic_waker::AtomicWaker;
use crate::windows::{Event, RawEvent};
use crate::Result;
use std::io;
use std::mem;
use windows_sys::Windows::Win32::Foundation as f;
use windows_sys::Windows::Win32::System::Threading as th;
use windows_sys::Windows::Win32::System::WindowsProgramming as wp;

/// Data on the waker for a handle.
struct Waker {
    ready: AtomicBool,
    waker: AtomicWaker,
    handle: f::HANDLE,
}

struct Shared {
    running: AtomicBool,
    holders: Mutex<Holders>,
    parker: Event,
}

#[derive(Default)]
struct Holders {
    added: Vec<Arc<Waker>>,
    removed: Vec<Event>,
}

/// An executor to drive things which are woken up by [windows event
/// objects].
///
/// This is necessary to use in combination with [AsyncEvent].
///
/// [windows event objects]:
/// https://docs.microsoft.com/en-us/windows/win32/sync/event-objects
pub struct EventsDriver {
    thread: Option<thread::JoinHandle<()>>,
    shared: Arc<Shared>,
}

impl EventsDriver {
    /// Construct a new events windows event object driver and return its
    /// handle.
    pub fn new() -> Result<Self> {
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
            self.shared.parker.set();

            if thread.join().is_err() {
                panic!("event handler thread panicked");
            }
        }
    }
}

impl Drop for EventsDriver {
    fn drop(&mut self) {
        let _ = self.inner_join();
    }
}

struct Driver {
    events: Vec<f::HANDLE>,
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
                th::WaitForMultipleObjects(
                    self.events.len() as u32,
                    self.events.as_ptr(),
                    false,
                    wp::INFINITE,
                )
            };

            match result {
                th::WAIT_ABANDONED_0 => panic!("wait abandoned"),
                th::WAIT_TIMEOUT => panic!("timed out"),
                th::WAIT_FAILED => {
                    panic!("wait failed: {}", io::Error::last_os_error())
                }
                other => {
                    let base = th::WAIT_OBJECT_0.0;
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

            let mut holders = self.shared.holders.lock();
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

/// An asynchronous variant of [Event].
///
/// See [AsyncEvent::new].
pub struct AsyncEvent {
    shared: Arc<Shared>,
    waker: Arc<Waker>,
    event: Option<Event>,
}

impl AsyncEvent {
    /// Construct an asynchronous event associated with the current handle. The
    /// constructed event has the initial state specified by `initial_state`.
    ///
    /// # Panics
    ///
    /// Panics unless an audio runtime is available.
    ///
    /// See [Runtime][crate::runtime::Runtime].
    pub fn new(initial_state: bool) -> windows::Result<AsyncEvent> {
        crate::runtime::with_events(|events| {
            let event = Event::new(false, initial_state)?;
            let handle = unsafe { event.raw_event() };

            let waker = Arc::new(Waker {
                ready: AtomicBool::new(false),
                waker: AtomicWaker::new(),
                handle,
            });

            events.shared.holders.lock().added.push(waker.clone());
            events.shared.parker.set();

            Ok(AsyncEvent {
                shared: events.shared.clone(),
                waker,
                event: Some(event),
            })
        })
    }

    /// Wait for the specified event handle to become set.
    pub async fn wait(&self) {
        use std::future::Future;
        use std::pin::Pin;
        use std::task::{Context, Poll};

        return WaitFor {
            shared: &*self.shared,
            waker: &*self.waker,
        }
        .await;

        struct WaitFor<'a> {
            shared: &'a Shared,
            waker: &'a Waker,
        }

        impl Future for WaitFor<'_> {
            type Output = ();

            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                if !self.shared.running.load(Ordering::Acquire) {
                    panic!("background thread panicked");
                }

                if self.waker.ready.load(Ordering::Acquire) {
                    return Poll::Ready(());
                }

                self.waker.waker.register_by_ref(cx.waker());
                Poll::Pending
            }
        }
    }

    /// Set the current event handle.
    pub fn set(&self) {
        self.event.as_ref().unwrap().set();
    }
}

impl RawEvent for AsyncEvent {
    unsafe fn raw_event(&self) -> f::HANDLE {
        self.event.as_ref().unwrap().raw_event()
    }
}

impl Drop for AsyncEvent {
    fn drop(&mut self) {
        let event = self.event.take().unwrap();
        self.shared.holders.lock().removed.push(event);
        self.shared.parker.set();
    }
}

unsafe impl Send for AsyncEvent {}
unsafe impl Sync for AsyncEvent {}
