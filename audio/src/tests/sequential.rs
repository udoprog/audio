/// Note: most of these tests are duplicated doc tests, but they're here so that
/// we can run them through miri and get a good idea of the soundness of our
/// implementations.

#[test]
fn test_channels_then_resize() {
    let mut buffer = crate::Sequential::<f32>::new();

    buffer.resize_channels(4);
    buffer.resize(128);

    let expected = vec![0.0; 128];

    assert_eq!(buffer.get(0).unwrap(), &expected[..]);
    assert_eq!(buffer.get(1).unwrap(), &expected[..]);
    assert_eq!(buffer.get(2).unwrap(), &expected[..]);
    assert_eq!(buffer.get(3).unwrap(), &expected[..]);
    assert_eq!(None, buffer.get(4));
}

#[test]
fn test_resize_then_channels() {
    let mut buffer = crate::Sequential::<f32>::new();

    buffer.resize(128);
    buffer.resize_channels(4);

    let expected = vec![0.0; 128];

    assert_eq!(buffer.get(0).unwrap(), &expected[..]);
    assert_eq!(buffer.get(1).unwrap(), &expected[..]);
    assert_eq!(buffer.get(2).unwrap(), &expected[..]);
    assert_eq!(buffer.get(3).unwrap(), &expected[..]);
    assert!(buffer.get(4).is_none());
}

#[test]
fn test_empty_channels() {
    let mut buffer = crate::Sequential::<f32>::new();

    buffer.resize_channels(4);

    assert!(buffer.get(0).is_some());
    assert!(buffer.get(1).is_some());
    assert!(buffer.get(2).is_some());
    assert!(buffer.get(3).is_some());
    assert!(buffer.get(4).is_none());
}

#[test]
fn test_empty() {
    let buffer = crate::Sequential::<f32>::new();

    assert_eq!(buffer.frames(), 0);
    assert!(buffer.get(0).is_none());
}

#[test]
fn test_multiple_resizes() {
    let mut buffer = crate::Sequential::<f32>::new();

    buffer.resize_channels(4);
    buffer.resize(128);

    let expected = vec![0.0; 128];

    assert_eq!(buffer.get(0).unwrap(), &expected[..]);
    assert_eq!(buffer.get(1).unwrap(), &expected[..]);
    assert_eq!(buffer.get(2).unwrap(), &expected[..]);
    assert_eq!(buffer.get(3).unwrap(), &expected[..]);
    assert!(buffer.get(4).is_none());
}

#[test]
fn test_unaligned_resize() {
    let mut buffer = crate::Sequential::<f32>::with_topology(2, 4);
    buffer[0].copy_from_slice(&[1.0, 2.0, 3.0, 4.0]);
    buffer[1].copy_from_slice(&[2.0, 3.0, 4.0, 5.0]);

    buffer.resize(3);

    assert_eq!(&buffer[0], &[1.0, 2.0, 3.0]);
    assert_eq!(&buffer[1], &[2.0, 3.0, 4.0]);

    buffer.resize(4);

    assert_eq!(&buffer[0], &[1.0, 2.0, 3.0, 2.0]); // <- 2.0 is stale data.
    assert_eq!(&buffer[1], &[2.0, 3.0, 4.0, 5.0]); // <- 5.0 is stale data.
}

#[test]
fn test_multiple_channel_resizes() {
    let mut buffer = crate::Sequential::<f32>::new();

    buffer.resize_channels(4);
    buffer.resize(128);

    let expected = (0..128).map(|v| v as f32).collect::<Vec<_>>();

    for mut chan in buffer.iter_mut() {
        for (s, v) in chan.iter_mut().zip(&expected) {
            *s = *v;
        }
    }

    assert_eq!(buffer.get(0).unwrap(), &expected[..]);
    assert_eq!(buffer.get(1).unwrap(), &expected[..]);
    assert_eq!(buffer.get(2).unwrap(), &expected[..]);
    assert_eq!(buffer.get(3).unwrap(), &expected[..]);
    assert!(buffer.get(4).is_none());

    buffer.resize_channels(2);

    assert_eq!(buffer.get(0).unwrap(), &expected[..]);
    assert_eq!(buffer.get(1).unwrap(), &expected[..]);
    assert!(buffer.get(2).is_none());

    // shrink
    buffer.resize(64);

    assert_eq!(buffer.get(0).unwrap(), &expected[..64]);
    assert_eq!(buffer.get(1).unwrap(), &expected[..64]);
    assert!(buffer.get(2).is_none());

    // increase - this causes some weirdness.
    buffer.resize(128);

    let first_overlapping = expected[..64]
        .iter()
        .chain(expected[..64].iter())
        .copied()
        .collect::<Vec<_>>();

    assert_eq!(buffer.get(0).unwrap(), &first_overlapping[..]);
    // Note: second channel matches perfectly up with an old channel that was
    // masked out.
    assert_eq!(buffer.get(1).unwrap(), &expected[..]);
    assert!(buffer.get(2).is_none());
}

#[test]
fn test_drop_empty() {
    let mut buffer = crate::Sequential::<f32>::new();

    assert_eq!(buffer.frames(), 0);
    buffer.resize(128);
    assert_eq!(buffer.frames(), 128);
}

#[test]
fn test_stale_allocation() {
    let mut buffer = crate::Sequential::<f32>::with_topology(4, 256);
    assert_eq!(buffer[1][128], 0.0);
    buffer[1][128] = 42.0;

    buffer.resize(64);
    assert!(buffer[1].get(128).is_none());

    buffer.resize(256);
    assert_eq!(buffer[1][128], 0.0);
}

#[test]
fn test_from_array() {
    let _ = crate::dynamic![[0.0; 128]; 2];
}
