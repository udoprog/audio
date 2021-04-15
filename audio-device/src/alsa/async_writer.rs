use crate::alsa::{Error, Pcm, Result};
use crate::libc as c;
use crate::unix::errno;
use audio_core as core;
use std::io;
use std::marker;
use std::os::unix::io::RawFd;
use tokio::io::unix::AsyncFd;

/// An interleaved type-checked async PCM writer.
///
/// See [Pcm::async_writer].
pub struct AsyncWriter<'a, T> {
    pcm: &'a mut Pcm,
    fd: AsyncFd<RawFd>,
    poll_fd: c::pollfd,
    channels: usize,
    _marker: marker::PhantomData<T>,
}

impl<'a, T> AsyncWriter<'a, T> {
    /// Construct a new writer surrounding the given PCM.
    ///
    /// # Safety
    ///
    /// This constructor assumes that the caller has checked that type `T` is
    /// appropriate for writing to the given PCM.
    pub(super) unsafe fn new(
        pcm: &'a mut Pcm,
        poll_fd: c::pollfd,
        channels: usize,
    ) -> io::Result<Self> {
        Ok(Self {
            pcm,
            fd: AsyncFd::new(poll_fd.fd)?,
            poll_fd,
            channels,
            _marker: marker::PhantomData,
        })
    }

    /// Write an interleaved buffer.
    pub async fn write_interleaved<B>(&mut self, mut buf: B) -> Result<()>
    where
        B: core::ReadBuf + core::ExactSizeBuf + core::AsInterleaved<T>,
    {
        if buf.channels() != self.channels {
            return Err(Error::ChannelsMismatch {
                actual: buf.channels(),
                expected: self.channels,
            });
        }

        while buf.has_remaining() {
            self.pcm.tag.ensure_on_thread();
            let frames = buf.frames() as usize;

            unsafe {
                let result = {
                    let ptr = buf.as_interleaved().as_ptr() as *const c::c_void;
                    self.pcm.write_interleaved_unchecked(ptr, frames as u64)
                };

                if result < 0 {
                    match errno::Errno::from_i32(-result as i32) {
                        errno::EWOULDBLOCK => {
                            let _ = self.fd.readable().await?;
                            self.poll_fd.revents = c::POLLIN;
                            dbg!(self.poll_fd);

                            let mut fds = [self.poll_fd];
                            let flags = self.pcm.poll_descriptors_revents(&mut fds)?;
                            println!("demangled = {:?}", flags);

                            // let _ = self.io.writable().await?;
                            continue;
                        }
                        errno => return Err(Error::Sys(errno)),
                    }
                }

                buf.advance(result as usize);
            }
        }

        Ok(())
    }
}

// Safety: [Pcm] is tagged with the thread its created it and is ensured not to
// leave it.
unsafe impl<T> Send for AsyncWriter<'_, T> {}
