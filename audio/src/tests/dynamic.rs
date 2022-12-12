/// Note: most of these tests are duplicated doc tests, but they're here so that
/// we can run them through miri and get a good idea of the soundness of our
/// implementations.

#[test]
fn test_channels_then_resize() {
    let mut buf = crate::buf::Dynamic::<f32>::new();

    buf.resize_channels(4);
    buf.resize_frames(1024);

    let expected = vec![0.0; 1024];

    assert_eq!(buf.get_channel(0).unwrap(), &expected[..]);
    assert_eq!(buf.get_channel(1).unwrap(), &expected[..]);
    assert_eq!(buf.get_channel(2).unwrap(), &expected[..]);
    assert_eq!(buf.get_channel(3).unwrap(), &expected[..]);
    assert!(buf.get_channel(4).is_none());
}

#[test]
fn test_resize_then_channels() {
    let mut buf = crate::buf::Dynamic::<f32>::new();

    buf.resize_frames(1024);
    buf.resize_channels(4);

    let expected = vec![0.0; 1024];

    assert_eq!(buf.get_channel(0).unwrap(), &expected[..]);
    assert_eq!(buf.get_channel(1).unwrap(), &expected[..]);
    assert_eq!(buf.get_channel(2).unwrap(), &expected[..]);
    assert_eq!(buf.get_channel(3).unwrap(), &expected[..]);
    assert!(buf.get_channel(4).is_none());
}

#[test]
fn test_empty_channels() {
    let mut buf = crate::buf::Dynamic::<f32>::new();

    buf.resize_channels(4);

    assert!(buf.get_channel(0).is_some());
    assert!(buf.get_channel(1).is_some());
    assert!(buf.get_channel(2).is_some());
    assert!(buf.get_channel(3).is_some());
    assert!(buf.get_channel(4).is_none());
}

#[test]
fn test_empty() {
    let buf = crate::buf::Dynamic::<f32>::new();

    assert_eq!(buf.frames(), 0);
    assert!(buf.get_channel(0).is_none());
}

#[test]
fn test_multiple_resizes() {
    let mut buf = crate::buf::Dynamic::<f32>::new();

    buf.resize_channels(4);
    buf.resize_frames(1024);

    let expected = vec![0.0; 1024];

    assert_eq!(buf.get_channel(0).unwrap(), &expected[..]);
    assert_eq!(buf.get_channel(1).unwrap(), &expected[..]);
    assert_eq!(buf.get_channel(2).unwrap(), &expected[..]);
    assert_eq!(buf.get_channel(3).unwrap(), &expected[..]);
    assert!(buf.get_channel(4).is_none());
}

#[test]
fn test_multiple_channel_resizes() {
    let mut buf = crate::buf::Dynamic::<f32>::new();

    buf.resize_channels(4);
    buf.resize_frames(1024);

    let expected = vec![0.0f32; 1024];

    assert_eq!(buf.get_channel(0).unwrap(), &expected[..]);
    assert_eq!(buf.get_channel(1).unwrap(), &expected[..]);
    assert_eq!(buf.get_channel(2).unwrap(), &expected[..]);
    assert_eq!(buf.get_channel(3).unwrap(), &expected[..]);
    assert!(buf.get_channel(4).is_none());

    buf.resize_channels(2);

    assert_eq!(buf.get_channel(0).unwrap(), &expected[..]);
    assert_eq!(buf.get_channel(1).unwrap(), &expected[..]);
    assert!(buf.get_channel(2).is_none());
    assert!(buf.get_channel(3).is_none());
    assert!(buf.get_channel(4).is_none());
}

#[test]
fn test_drop_empty() {
    let mut buf = crate::buf::Dynamic::<f32>::new();

    assert_eq!(buf.frames(), 0);
    buf.resize_frames(1024);
    assert_eq!(buf.frames(), 1024);
}

#[test]
fn test_into_vecs() {
    let mut buf = crate::buf::Dynamic::<f32>::new();
    buf.resize_channels(4);
    buf.resize_frames(512);

    let expected = vec![0.0; 512];

    let buffers = buf.into_vectors();
    assert_eq!(buffers.len(), 4);
    assert_eq!(buffers[0], &expected[..]);
    assert_eq!(buffers[1], &expected[..]);
    assert_eq!(buffers[2], &expected[..]);
    assert_eq!(buffers[3], &expected[..]);
}

#[test]
fn test_enabled_mut() {
    use bittle::Mask as _;

    let mut buf = crate::buf::Dynamic::<f32>::with_topology(4, 1024);
    let mask: bittle::FixedSet<u128> = bittle::fixed_set![0, 2, 3];

    for mut chan in mask.join(buf.iter_channels_mut()) {
        for b in chan.iter_mut() {
            *b = 1.0;
        }
    }

    let zeroed = vec![0.0f32; 1024];
    let expected = vec![1.0f32; 1024];

    assert_eq!(&buf[0], &expected[..]);
    assert_eq!(&buf[1], &zeroed[..]);
    assert_eq!(&buf[2], &expected[..]);
    assert_eq!(&buf[3], &expected[..]);
}

#[test]
fn test_stale_allocation() {
    let mut buf = crate::buf::Dynamic::<f32>::with_topology(4, 256);
    assert_eq!(buf[1][128], 0.0);
    buf[1][128] = 42.0;

    buf.resize_frames(64);
    assert!(buf[1].get(128).is_none());

    buf.resize_frames(256);
    assert_eq!(buf[1][128], 42.0);
}

#[test]
fn test_from_array() {
    let _ = crate::dynamic![[0.0; 1024]; 2];
}

#[test]
fn test_get_mut() {
    use rand::Rng as _;

    let mut buf = crate::buf::Dynamic::<f32>::new();

    buf.resize_channels(2);
    buf.resize_frames(256);

    let mut rng = rand::thread_rng();

    if let Some(mut left) = buf.get_mut(0) {
        rng.fill(left.as_mut());
    }

    if let Some(mut right) = buf.get_mut(1) {
        rng.fill(right.as_mut());
    }
}

#[test]
fn test_get_or_default() {
    let mut buf = crate::buf::Dynamic::<f32>::new();

    buf.resize_frames(256);

    let expected = vec![0f32; 256];

    assert_eq!(buf.get_or_default(0), &expected[..]);
    assert_eq!(buf.get_or_default(1), &expected[..]);

    assert_eq!(buf.channels(), 2);
}

#[test]
fn test_resize_topology() {
    let mut buf = crate::buf::Dynamic::<f64>::with_topology(1, 1024);

    buf.resize_frames(20480);
    buf.resize_channels(1);
}
