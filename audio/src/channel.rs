//! Channel buffers.
//!
//! * [LinearMut] and [LinearRef] wraps a mutable and immutable *linear* channel
//!   buffer respectively.
//! * [InterleavedMut] and [InterleavedRef] wraps mutable and immutable
//!   *interleaved* channel buffers respectively.

pub mod linear;
pub use self::linear::{LinearMut, LinearRef};

pub mod interleaved;
pub use self::interleaved::{InterleavedMut, InterleavedRef};

use core::{Channel, ChannelMut};

/// Copy the content of one channel to another.
///
/// # Examples
///
/// ```
/// use audio::{Buf, BufMut, ChannelMut};
///
/// let from = audio::interleaved![[1i16; 4]; 2];
/// let mut to = audio::buf::Interleaved::<i16>::with_topology(2, 4);
///
/// audio::channel::copy(from.limit(2).get(0).unwrap(), to.get_mut(0).unwrap());
/// assert_eq!(to.as_slice(), &[1, 0, 1, 0, 0, 0, 0, 0]);
/// ```
pub fn copy<I, O>(from: I, mut to: O)
where
    I: Channel,
    O: ChannelMut<Sample = I::Sample>,
    I::Sample: Copy,
{
    match (from.try_as_linear(), to.try_as_linear_mut()) {
        (Some(from), Some(to)) => {
            let len = usize::min(to.len(), from.len());
            to[..len].copy_from_slice(&from[..len]);
        }
        _ => {
            for (t, f) in to.iter_mut().zip(from.iter()) {
                *t = f;
            }
        }
    }
}

/// Copy an iterator into a channel.
///
/// # Examples
///
/// ```
/// use audio::ChannelMut;
///
/// let mut to = audio::buf::Interleaved::<i16>::with_topology(2, 4);
///
/// audio::channel::copy_iter(0i16.., to.get_mut(0).unwrap());
/// assert_eq!(to.as_slice(), &[0, 0, 1, 0, 2, 0, 3, 0]);
/// ```
pub fn copy_iter<I, O>(from: I, mut to: O)
where
    I: IntoIterator,
    O: ChannelMut<Sample = I::Item>,
    I::Item: Copy,
{
    for (t, f) in to.iter_mut().zip(from) {
        *t = f;
    }
}
