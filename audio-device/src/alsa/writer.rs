use crate::alsa::{Error, Pcm, Result};
use crate::libc as c;
use audio_core as core;
use std::marker;

/// A interleaved type-checked PCM writer.
///
/// See [Pcm::writer].
pub struct Writer<'a, T> {
    pcm: &'a mut Pcm,
    channels: usize,
    _marker: marker::PhantomData<T>,
}

impl<'a, T> Writer<'a, T> {
    /// Construct a new writer surrounding the given PCM.
    ///
    /// # Safety
    ///
    /// This constructor assumes that the caller has checked that type `T` is
    /// appropriate for writing to the given PCM.
    pub(super) unsafe fn new(pcm: &'a mut Pcm, channels: usize) -> Self {
        Self {
            pcm,
            channels,
            _marker: marker::PhantomData,
        }
    }

    /// Write an interleaved buffer.
    pub fn write_interleaved<B>(&mut self, mut buf: B) -> Result<()>
    where
        B: core::ReadBuf + core::ExactSizeBuf + core::AsInterleaved<T>,
    {
        if buf.channels() != self.channels {
            return Err(Error::ChannelsMismatch {
                actual: buf.channels(),
                expected: self.channels,
            });
        }

        let frames = buf.frames() as usize;

        unsafe {
            let ptr = buf.as_interleaved().as_ptr() as *const c::c_void;
            let written = self.pcm.write_interleaved_unchecked(ptr, frames as u64)?;
            buf.advance(written as usize);
        }

        Ok(())
    }
}
