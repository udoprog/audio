use std::io;
use std::marker;
use std::mem;
use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::mpsc;
use std::thread;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("audio thread panicked")]
pub struct Panicked(());

struct Submit {
    /// The thread to be woken up once the operation is completed.
    thread: thread::Thread,
    /// Where the result of the operation is written back out into. The value
    /// is null as long as the thread has not completed.
    storage: *mut AtomicPtr<()>,
    /// The boxed function which performs the operation.
    task: Box<dyn FnMut(thread::Thread, *mut AtomicPtr<()>) + Send + 'static>,
}

enum Task {
    Submit(Submit),
    Join,
}

unsafe impl Send for Task {}

/// Handle to a background audio thread, suitable for running audio-related
/// operations.
#[must_use = "The audio thread should be joined with AudioThread::join once no longer used"]
pub struct AudioThread {
    /// Things that have been submitted for execution on the audio thread.
    tx: mpsc::SyncSender<Task>,
    /// The handle associated with the audio thread.
    handle: thread::JoinHandle<()>,
}

impl AudioThread {
    /// Construct a new background audio thread.
    pub fn new() -> io::Result<Self> {
        let (tx, rx) = mpsc::sync_channel(0);

        let handle = thread::Builder::new()
            .name(String::from("audio-thread"))
            .spawn(move || Self::worker(rx))?;

        Ok(Self { tx, handle })
    }

    /// Submit a task to run on the background audio thread.
    pub fn submit<F, T>(&self, task: F) -> Result<T, Panicked>
    where
        F: 'static + Send + FnOnce() -> T,
        T: 'static + Send,
    {
        let thread = thread::current();
        let storage = Box::into_raw(Box::new(AtomicPtr::new(ptr::null_mut())));

        let result = self.tx.send(Task::Submit(Submit {
            thread: thread.clone(),
            storage,
            task: Box::new(into_task(task)),
        }));

        if result.is_err() {
            // NB: free return address on errors.
            let _ = unsafe { Box::from_raw(storage) };
            return Err(Panicked(()));
        }

        // Park until a result is available.
        loop {
            thread::park();

            // Try and load storage, if not set yet we continue spinning
            // (spurious wake).
            //
            // Safety: we're the only ones controlling these, so we know that
            // they are correctly allocated and who owns what with
            // synchronization.
            unsafe {
                let result = (*storage).load(Ordering::Acquire);

                if result.is_null() {
                    continue;
                }

                let _ = Box::from_raw(storage);

                return match *Box::from_raw(result as *mut Option<T>) {
                    Some(result) => Ok(result),
                    None => Err(Panicked(())),
                };
            }
        }

        fn into_task<F, T>(
            task: F,
        ) -> impl FnMut(thread::Thread, *mut AtomicPtr<()>) + 'static + Send
        where
            F: FnOnce() -> T + 'static + Send,
        {
            let mut task = Some(task);

            return move |thread, storage| {
                let guard: SubmitGuard<T> = SubmitGuard {
                    thread,
                    storage,
                    _marker: marker::PhantomData,
                };

                let task = task.take().expect("task has already been consumed");
                let output = task();
                let guard = mem::ManuallyDrop::new(guard);

                let output = Box::into_raw(Box::new(Some(output)));

                // Safety: we're the only one with synchronized access to this
                // pointer, and we know it hasn't been de-allocated yet.
                unsafe {
                    (*guard.storage).store(output as *mut (), Ordering::Release);
                }

                guard.thread.unpark();
            };

            struct SubmitGuard<T> {
                thread: thread::Thread,
                storage: *mut AtomicPtr<()>,
                _marker: marker::PhantomData<T>,
            }

            impl<T> Drop for SubmitGuard<T> {
                fn drop(&mut self) {
                    let output = Box::into_raw(Box::new(Option::<T>::None));

                    // Safety: We free the pointer if we are unwinding due to a
                    // panic.
                    //
                    // We know this is safe, because only user-provided code can
                    // panic, and anything surrounding it is panic-safe. We disarm
                    // the guard *after* user-provided code has executed.
                    unsafe {
                        (*self.storage).store(output as *mut (), Ordering::Release);
                    }

                    self.thread.unpark();
                }
            }
        }
    }

    /// Join the audio background thread.
    pub fn join(self) -> Result<(), Panicked> {
        // Thread has panicked.
        if self.tx.send(Task::Join).is_err() {
            return self.handle.join().map_err(|_| Panicked(()));
        }

        self.handle.thread().unpark();
        self.handle.join().map_err(|_| Panicked(()))
    }

    /// Worker thread.
    fn worker(rx: mpsc::Receiver<Task>) {
        #[cfg(windows)]
        if let Err(e) = windows::initialize_mta() {
            panic!("windows: failed to initialize windows mta: {}", e);
        }

        loop {
            let task = rx.recv().unwrap();

            match task {
                Task::Submit(mut submit) => {
                    (submit.task)(submit.thread, submit.storage);
                    continue;
                }
                Task::Join => break,
            }
        }
    }
}
