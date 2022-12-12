/// Note: most of these tests are duplicated doc tests, but they're here so that
/// we can run them through miri and get a good idea of the soundness of our
/// implementations.

#[test]
fn test_channels_then_resize() {
    let mut buf = crate::buf::Sequential::<f32>::new();

    buf.resize_channels(4);
    buf.resize_frames(128);

    let expected = vec![0.0; 128];

    assert_eq!(buf.get_channel(0).unwrap(), &expected[..]);
    assert_eq!(buf.get_channel(1).unwrap(), &expected[..]);
    assert_eq!(buf.get_channel(2).unwrap(), &expected[..]);
    assert_eq!(buf.get_channel(3).unwrap(), &expected[..]);
    assert_eq!(None, buf.get_channel(4));
}

#[test]
fn test_resize_then_channels() {
    let mut buf = crate::buf::Sequential::<f32>::new();

    buf.resize_frames(128);
    buf.resize_channels(4);

    let expected = vec![0.0; 128];

    assert_eq!(buf.get_channel(0).unwrap(), &expected[..]);
    assert_eq!(buf.get_channel(1).unwrap(), &expected[..]);
    assert_eq!(buf.get_channel(2).unwrap(), &expected[..]);
    assert_eq!(buf.get_channel(3).unwrap(), &expected[..]);
    assert!(buf.get_channel(4).is_none());
}

#[test]
fn test_empty_channels() {
    let mut buf = crate::buf::Sequential::<f32>::new();

    buf.resize_channels(4);

    assert!(buf.get_channel(0).is_some());
    assert!(buf.get_channel(1).is_some());
    assert!(buf.get_channel(2).is_some());
    assert!(buf.get_channel(3).is_some());
    assert!(buf.get_channel(4).is_none());
}

#[test]
fn test_empty() {
    let buf = crate::buf::Sequential::<f32>::new();

    assert_eq!(buf.frames(), 0);
    assert!(buf.get_channel(0).is_none());
}

#[test]
fn test_multiple_resizes() {
    let mut buf = crate::buf::Sequential::<f32>::new();

    buf.resize_channels(4);
    buf.resize_frames(128);

    let expected = vec![0.0; 128];

    assert_eq!(buf.get_channel(0).unwrap(), &expected[..]);
    assert_eq!(buf.get_channel(1).unwrap(), &expected[..]);
    assert_eq!(buf.get_channel(2).unwrap(), &expected[..]);
    assert_eq!(buf.get_channel(3).unwrap(), &expected[..]);
    assert!(buf.get_channel(4).is_none());
}

#[test]
fn test_unaligned_resize() {
    let mut buf = crate::buf::Sequential::<f32>::with_topology(2, 4);
    buf[0].copy_from_slice(&[1.0, 2.0, 3.0, 4.0]);
    buf[1].copy_from_slice(&[2.0, 3.0, 4.0, 5.0]);

    buf.resize_frames(3);

    assert_eq!(&buf[0], &[1.0, 2.0, 3.0]);
    assert_eq!(&buf[1], &[2.0, 3.0, 4.0]);

    buf.resize_frames(4);

    assert_eq!(&buf[0], &[1.0, 2.0, 3.0, 2.0]); // <- 2.0 is stale data.
    assert_eq!(&buf[1], &[2.0, 3.0, 4.0, 5.0]); // <- 5.0 is stale data.
}

#[test]
fn test_multiple_channel_resizes() {
    let mut buf = crate::buf::Sequential::<f32>::new();

    buf.resize_channels(4);
    buf.resize_frames(128);

    let expected = (0..128).map(|v| v as f32).collect::<Vec<_>>();

    for mut chan in buf.iter_channels_mut() {
        for (s, v) in chan.iter_mut().zip(&expected) {
            *s = *v;
        }
    }

    assert_eq!(buf.get_channel(0).unwrap(), &expected[..]);
    assert_eq!(buf.get_channel(1).unwrap(), &expected[..]);
    assert_eq!(buf.get_channel(2).unwrap(), &expected[..]);
    assert_eq!(buf.get_channel(3).unwrap(), &expected[..]);
    assert!(buf.get_channel(4).is_none());

    buf.resize_channels(2);

    assert_eq!(buf.get_channel(0).unwrap(), &expected[..]);
    assert_eq!(buf.get_channel(1).unwrap(), &expected[..]);
    assert!(buf.get_channel(2).is_none());

    // shrink
    buf.resize_frames(64);

    assert_eq!(buf.get_channel(0).unwrap(), &expected[..64]);
    assert_eq!(buf.get_channel(1).unwrap(), &expected[..64]);
    assert!(buf.get_channel(2).is_none());

    // increase - this causes some weirdness.
    buf.resize_frames(128);

    let first_overlapping = expected[..64]
        .iter()
        .chain(expected[..64].iter())
        .copied()
        .collect::<Vec<_>>();

    assert_eq!(buf.get_channel(0).unwrap(), &first_overlapping[..]);
    // Note: second channel matches perfectly up with an old channel that was
    // masked out.
    assert_eq!(buf.get_channel(1).unwrap(), &expected[..]);
    assert!(buf.get_channel(2).is_none());
}

#[test]
fn test_drop_empty() {
    let mut buf = crate::buf::Sequential::<f32>::new();

    assert_eq!(buf.frames(), 0);
    buf.resize_frames(128);
    assert_eq!(buf.frames(), 128);
}

#[test]
fn test_stale_allocation() {
    let mut buf = crate::buf::Sequential::<f32>::with_topology(4, 256);
    assert_eq!(buf[1][128], 0.0);
    buf[1][128] = 42.0;

    buf.resize_frames(64);
    assert!(buf[1].get(128).is_none());

    buf.resize_frames(256);
    assert_eq!(buf[1][128], 0.0);
}

#[test]
fn test_from_array() {
    let _ = crate::dynamic![[0.0; 128]; 2];
}
