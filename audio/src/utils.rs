use core::ptr::{self, NonNull};

/// Utility functions to copy a channel in-place in a sequential audio buffer
/// from one place to another.
///
/// # Safety
///
/// Caller has to ensure that the `data` pointer points to sequential memory
/// with the correct topology and initialized frame count.
pub(crate) unsafe fn copy_channels_sequential<T>(
    data: NonNull<T>,
    channels: usize,
    frames: usize,
    from: usize,
    to: usize,
) where
    T: Copy,
{
    assert! {
        from < channels,
        "copy from channel {} is out of bounds 0-{}",
        from,
        channels
    };
    assert! {
        to < channels,
        "copy to channel {} which is out of bounds 0-{}",
        to,
        channels
    };

    unsafe {
        if from != to {
            let from = data.as_ptr().add(from * frames);
            let to = data.as_ptr().add(to * frames);
            ptr::copy_nonoverlapping(from, to, frames);
        }
    }
}

/// Utility functions to copy a channel in-place in an interleaved audio buffer
/// from one place to another.
///
/// # Safety
///
/// Caller has to ensure that the `data` pointer points to interleaved memory
/// with the correct topology and initialized frame count.
pub(crate) unsafe fn copy_channels_interleaved<T>(
    data: NonNull<T>,
    channels: usize,
    frames: usize,
    from: usize,
    to: usize,
) where
    T: Copy,
{
    assert! {
        from < channels,
        "copy from channel {} is out of bounds 0-{}",
        from,
        channels
    };
    assert! {
        to < channels,
        "copy to channel {} which is out of bounds 0-{}",
        to,
        channels
    };

    unsafe {
        if from != to {
            // Safety: We're making sure not to access any mutable buffers which
            // are not initialized.
            for n in 0..frames {
                let from = data.as_ptr().add(from + channels * n) as *const _;
                let to = data.as_ptr().add(to + channels * n);
                let f = ptr::read(from);
                ptr::write(to, f);
            }
        }
    }
}
