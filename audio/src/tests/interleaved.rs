/// Note: most of these tests are duplicated doc tests, but they're here so that
/// we can run them through miri and get a good idea of the soundness of our
/// implementations.
use audio_core::{InterleavedBufMut, ResizableBuf};

#[test]
fn test_init() {
    let mut buf = crate::buf::Interleaved::<f32>::with_topology(2, 4);

    for (c, s) in buf
        .get_mut(0)
        .unwrap()
        .iter_mut()
        .zip(&[1.0, 2.0, 3.0, 4.0])
    {
        *c = *s;
    }

    for (c, s) in buf
        .get_mut(1)
        .unwrap()
        .iter_mut()
        .zip(&[5.0, 6.0, 7.0, 8.0])
    {
        *c = *s;
    }

    assert_eq!(buf.as_slice(), &[1.0, 5.0, 2.0, 6.0, 3.0, 7.0, 4.0, 8.0]);
}

#[test]
fn test_complicated() {
    let mut buf = crate::buf::Interleaved::<f32>::with_topology(2, 4);

    let mut it = buf.iter_mut();

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

    assert_eq!(buf.as_slice(), &[1.0, 5.0, 2.0, 6.0, 3.0, 7.0, 4.0, 8.0]);
}

#[test]
fn test_iter() {
    let mut buf = crate::buf::Interleaved::<f32>::with_topology(2, 4);

    let mut it = buf.iter_mut();

    for (c, f) in it.next().unwrap().iter_mut().zip(&[1.0, 2.0, 3.0, 4.0]) {
        *c = *f;
    }

    for (c, f) in it.next().unwrap().iter_mut().zip(&[5.0, 6.0, 7.0, 8.0]) {
        *c = *f;
    }

    let channels = buf.iter().collect::<Vec<_>>();
    let left = channels[0].iter().collect::<Vec<_>>();
    let right = channels[1].iter().collect::<Vec<_>>();
    let left2 = channels[0].iter().collect::<Vec<_>>();
    let right2 = channels[1].iter().collect::<Vec<_>>();

    assert_eq!(left, &[1.0, 2.0, 3.0, 4.0]);
    assert_eq!(right, &[5.0, 6.0, 7.0, 8.0]);
    assert_eq!(left2, &[1.0, 2.0, 3.0, 4.0]);
    assert_eq!(right2, &[5.0, 6.0, 7.0, 8.0]);
}

#[test]
fn test_iter_mut() {
    let mut buf = crate::buf::Interleaved::<f32>::with_topology(2, 4);

    let mut it = buf.iter_mut();

    for (c, f) in it.next().unwrap().iter_mut().zip(&[1.0, 2.0, 3.0, 4.0]) {
        *c = *f;
    }

    for (c, f) in it.next().unwrap().iter_mut().zip(&[5.0, 6.0, 7.0, 8.0]) {
        *c = *f;
    }

    let mut it = buf.iter_mut();

    let mut left = it.next().unwrap();
    let mut right = it.next().unwrap();

    let left = left.iter_mut().collect::<Vec<_>>();
    let right = right.iter_mut().collect::<Vec<_>>();

    assert_eq!(left, &[&mut 1.0, &mut 2.0, &mut 3.0, &mut 4.0]);
    assert_eq!(right, &[&mut 5.0, &mut 6.0, &mut 7.0, &mut 8.0]);
}

#[test]
fn test_resize() {
    let mut buf = crate::buf::Interleaved::<f32>::new();

    assert_eq!(buf.channels(), 0);
    assert_eq!(buf.frames(), 0);

    buf.resize_channels(4);
    buf.resize(256);

    assert_eq!(buf.channels(), 4);
    assert_eq!(buf.frames(), 256);

    {
        let mut chan = buf.get_mut(1).unwrap();

        assert_eq!(chan.get(127), Some(0.0));
        *chan.get_mut(127).unwrap() = 42.0;
        assert_eq!(chan.get(127), Some(42.0));
    }

    buf.resize(128);
    assert_eq!(buf.sample(1, 127), Some(42.0));

    buf.resize(256);
    assert_eq!(buf.sample(1, 127), Some(42.0));

    buf.resize_channels(2);
    assert_eq!(buf.sample(1, 127), Some(42.0));

    buf.resize(64);
    assert_eq!(buf.sample(1, 127), None);
}

// Miri: Grabbing a mutable pointer out of a slice has many potential issues.
// This tests basic soundness of the process.

#[test]
fn test_as_interleaved_mut_ptr() {
    use std::ptr;

    unsafe fn fill_with_ones(buf: ptr::NonNull<i16>, len: usize) -> (usize, usize) {
        let buf = std::slice::from_raw_parts_mut(buf.as_ptr(), len);

        for (o, b) in buf.iter_mut().zip(std::iter::repeat(1)) {
            *o = b;
        }

        (2, len / 2)
    }

    fn test(mut buf: impl ResizableBuf + InterleavedBufMut<Sample = i16>) {
        assert!(buf.try_reserve(16));
        // Note: call fills the buf with ones.
        // Safety: We've initialized exactly 16 frames before calling this
        // function.
        let (channels, frames) = unsafe { fill_with_ones(buf.as_interleaved_mut_ptr(), 16) };
        buf.resize_topology(channels, frames);
    }

    let mut buf = crate::buf::Interleaved::new();
    test(&mut buf);

    assert_eq! {
        buf.get(0).unwrap().iter().collect::<Vec<_>>(),
        &[1, 1, 1, 1, 1, 1, 1, 1],
    };
    assert_eq! {
        buf.get(1).unwrap().iter().collect::<Vec<_>>(),
        &[1, 1, 1, 1, 1, 1, 1, 1],
    };
}
