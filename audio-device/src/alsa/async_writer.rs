use core::marker;

use crate::alsa::{Error, Pcm, Result};
use crate::libc as c;
use crate::unix::{Errno, PollFlags};
use crate::unix::AsyncPoll;

/// An interleaved type-checked async PCM writer.
///
/// See [Pcm::async_writer].
pub struct AsyncWriter<'a, T> {
    pcm: &'a mut Pcm,
    poll_handle: AsyncPoll,
    pollfd: c::pollfd,
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
    pub(super) unsafe fn new(pcm: &'a mut Pcm, pollfd: c::pollfd, channels: usize) -> Result<Self> {
        Ok(Self {
            pcm,
            poll_handle: AsyncPoll::new(pollfd)?,
            pollfd,
            channels,
            _marker: marker::PhantomData,
        })
    }

    /// Write an interleaved buffer.
    pub async fn write_interleaved<B>(&mut self, mut buf: B) -> Result<()>
    where
        B: audio_core::Buf<Sample = T> + audio_core::ReadBuf + audio_core::ExactSizeBuf + audio_core::InterleavedBuf,
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

                let written = match result {
                    Ok(written) => written as usize,
                    Err(Error::Sys(Errno::EWOULDBLOCK)) => {
                        loop {
                            let guard = self.poll_handle.returned_events().await;
                            self.pollfd.revents = guard.events();

                            let mut fds = [self.pollfd];
                            let flags = self.pcm.poll_descriptors_revents(&mut fds)?;

                            if flags.test(PollFlags::POLLOUT) {
                                break;
                            }

                            drop(guard);
                        }

                        continue;
                    }
                    Err(e) => return Err(e),
                };

                buf.advance(written);
            }
        }

        Ok(())
    }
}

// Safety: [Pcm] is tagged with the thread its created it and is ensured not to
// leave it.
unsafe impl<T> Send for AsyncWriter<'_, T> {}
