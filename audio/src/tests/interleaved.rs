/// Note: most of these tests are duplicated doc tests, but they're here so that
/// we can run them through miri and get a good idea of the soundness of our
/// implementations.
use audio_core::Channel;

#[test]
fn test_init() {
    let mut buffer = crate::Interleaved::<f32>::with_topology(2, 4);

    for (c, s) in buffer
        .get_mut(0)
        .unwrap()
        .iter_mut()
        .zip(&[1.0, 2.0, 3.0, 4.0])
    {
        *c = *s;
    }

    for (c, s) in buffer
        .get_mut(1)
        .unwrap()
        .iter_mut()
        .zip(&[5.0, 6.0, 7.0, 8.0])
    {
        *c = *s;
    }

    assert_eq!(buffer.as_slice(), &[1.0, 5.0, 2.0, 6.0, 3.0, 7.0, 4.0, 8.0]);
}

#[test]
fn test_complicated() {
    let mut buffer = crate::Interleaved::<f32>::with_topology(2, 4);

    let mut it = buffer.iter_mut();

    let mut left_chan = it.next().unwrap();
    let mut right_chan = it.next().unwrap();

    let left = left_chan.iter_mut().collect::<Vec<_>>();
    let right = right_chan.iter_mut().collect::<Vec<_>>();

    for (c, f) in left.into_iter().zip(&[1.0, 2.0, 3.0, 4.0]) {
        *c = *f;
    }

    for (c, f) in right.into_iter().zip(&[5.0, 6.0, 7.0, 8.0]) {
        *c = *f;
    }

    assert_eq!(buffer.as_slice(), &[1.0, 5.0, 2.0, 6.0, 3.0, 7.0, 4.0, 8.0]);
}

#[test]
fn test_iter() {
    let mut buffer = crate::Interleaved::<f32>::with_topology(2, 4);

    let mut it = buffer.iter_mut();

    for (c, f) in it.next().unwrap().iter_mut().zip(&[1.0, 2.0, 3.0, 4.0]) {
        *c = *f;
    }

    for (c, f) in it.next().unwrap().iter_mut().zip(&[5.0, 6.0, 7.0, 8.0]) {
        *c = *f;
    }

    let channels = buffer.iter().collect::<Vec<_>>();
    let left = channels[0].iter().collect::<Vec<_>>();
    let right = channels[1].iter().collect::<Vec<_>>();
    let left2 = channels[0].iter().collect::<Vec<_>>();
    let right2 = channels[1].iter().collect::<Vec<_>>();

    assert_eq!(left, &[&1.0, &2.0, &3.0, &4.0]);
    assert_eq!(right, &[&5.0, &6.0, &7.0, &8.0]);
    assert_eq!(left2, &[&1.0, &2.0, &3.0, &4.0]);
    assert_eq!(right2, &[&5.0, &6.0, &7.0, &8.0]);
}

#[test]
fn test_iter_mut() {
    let mut buffer = crate::Interleaved::<f32>::with_topology(2, 4);

    let mut it = buffer.iter_mut();

    for (c, f) in it.next().unwrap().iter_mut().zip(&[1.0, 2.0, 3.0, 4.0]) {
        *c = *f;
    }

    for (c, f) in it.next().unwrap().iter_mut().zip(&[5.0, 6.0, 7.0, 8.0]) {
        *c = *f;
    }

    let mut it = buffer.iter_mut();

    let mut left = it.next().unwrap();
    let mut right = it.next().unwrap();

    let left = left.iter_mut().collect::<Vec<_>>();
    let right = right.iter_mut().collect::<Vec<_>>();

    assert_eq!(left, &[&mut 1.0, &mut 2.0, &mut 3.0, &mut 4.0]);
    assert_eq!(right, &[&mut 5.0, &mut 6.0, &mut 7.0, &mut 8.0]);
}

#[test]
fn test_resize() {
    let mut buffer = crate::Interleaved::<f32>::new();

    assert_eq!(buffer.channels(), 0);
    assert_eq!(buffer.frames(), 0);

    buffer.resize_channels(4);
    buffer.resize(256);

    assert_eq!(buffer.channels(), 4);
    assert_eq!(buffer.frames(), 256);

    {
        let mut chan = buffer.get_mut(1).unwrap();

        assert_eq!(chan.get(127), Some(0.0));
        *chan.get_mut(127).unwrap() = 42.0;
        assert_eq!(chan.get(127), Some(42.0));
    }

    buffer.resize(128);
    assert_eq!(buffer.frame(1, 127), Some(42.0));

    buffer.resize(256);
    assert_eq!(buffer.frame(1, 127), Some(42.0));

    buffer.resize_channels(2);
    assert_eq!(buffer.frame(1, 127), Some(42.0));

    buffer.resize(64);
    assert_eq!(buffer.frame(1, 127), None);
}

// Miri: Grabbing a mutable pointer out of a slice has many potential issues.
// This tests basic soundness of the process.

#[test]
fn test_as_interleaved_mut_ptr() {
    use crate::{AsInterleavedMut, Channels, InterleavedBuf};

    unsafe fn fill_with_ones(buf: *mut i16, len: usize) -> (usize, usize) {
        let buf = std::slice::from_raw_parts_mut(buf, len);

        for (o, b) in buf.iter_mut().zip(std::iter::repeat(1)) {
            *o = b;
        }

        (2, len / 2)
    }

    fn test<B>(mut buffer: B)
    where
        B: InterleavedBuf + AsInterleavedMut<i16>,
    {
        buffer.reserve_frames(16);
        // Note: call fills the buffer with ones.
        // Safety: We've initialized exactly 16 frames before calling this
        // function.
        let (channels, frames) = unsafe { fill_with_ones(buffer.as_interleaved_mut_ptr(), 16) };
        buffer.set_topology(channels, frames);
    }

    let mut buf = crate::Interleaved::new();
    test(&mut buf);

    assert_eq! {
        buf.channel(0).iter().copied().collect::<Vec<_>>(),
        &[1, 1, 1, 1, 1, 1, 1, 1],
    };
    assert_eq! {
        buf.channel(1).iter().copied().collect::<Vec<_>>(),
        &[1, 1, 1, 1, 1, 1, 1, 1],
    };
}
