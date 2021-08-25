/// Note: most of these tests are duplicated doc tests, but they're here so that
/// we can run them through miri and get a good idea of the soundness of our
/// implementations.

#[test]
fn test_channels_then_resize() {
    let mut buffer = crate::Dynamic::<f32>::new();

    buffer.resize_channels(4);
    buffer.resize(1024);

    let expected = vec![0.0; 1024];

    assert_eq!(buffer.get(0).unwrap(), &expected[..]);
    assert_eq!(buffer.get(1).unwrap(), &expected[..]);
    assert_eq!(buffer.get(2).unwrap(), &expected[..]);
    assert_eq!(buffer.get(3).unwrap(), &expected[..]);
    assert!(buffer.get(4).is_none());
}

#[test]
fn test_resize_then_channels() {
    let mut buffer = crate::Dynamic::<f32>::new();

    buffer.resize(1024);
    buffer.resize_channels(4);

    let expected = vec![0.0; 1024];

    assert_eq!(buffer.get(0).unwrap(), &expected[..]);
    assert_eq!(buffer.get(1).unwrap(), &expected[..]);
    assert_eq!(buffer.get(2).unwrap(), &expected[..]);
    assert_eq!(buffer.get(3).unwrap(), &expected[..]);
    assert!(buffer.get(4).is_none());
}

#[test]
fn test_empty_channels() {
    let mut buffer = crate::Dynamic::<f32>::new();

    buffer.resize_channels(4);

    assert!(buffer.get(0).is_some());
    assert!(buffer.get(1).is_some());
    assert!(buffer.get(2).is_some());
    assert!(buffer.get(3).is_some());
    assert!(buffer.get(4).is_none());
}

#[test]
fn test_empty() {
    let buffer = crate::Dynamic::<f32>::new();

    assert_eq!(buffer.frames(), 0);
    assert!(buffer.get(0).is_none());
}

#[test]
fn test_multiple_resizes() {
    let mut buffer = crate::Dynamic::<f32>::new();

    buffer.resize_channels(4);
    buffer.resize(1024);

    let expected = vec![0.0; 1024];

    assert_eq!(buffer.get(0).unwrap(), &expected[..]);
    assert_eq!(buffer.get(1).unwrap(), &expected[..]);
    assert_eq!(buffer.get(2).unwrap(), &expected[..]);
    assert_eq!(buffer.get(3).unwrap(), &expected[..]);
    assert!(buffer.get(4).is_none());
}

#[test]
fn test_multiple_channel_resizes() {
    let mut buffer = crate::Dynamic::<f32>::new();

    buffer.resize_channels(4);
    buffer.resize(1024);

    let expected = vec![0.0f32; 1024];

    assert_eq!(buffer.get(0).unwrap(), &expected[..]);
    assert_eq!(buffer.get(1).unwrap(), &expected[..]);
    assert_eq!(buffer.get(2).unwrap(), &expected[..]);
    assert_eq!(buffer.get(3).unwrap(), &expected[..]);
    assert!(buffer.get(4).is_none());

    buffer.resize_channels(2);

    assert_eq!(buffer.get(0).unwrap(), &expected[..]);
    assert_eq!(buffer.get(1).unwrap(), &expected[..]);
    assert!(buffer.get(2).is_none());
    assert!(buffer.get(3).is_none());
    assert!(buffer.get(4).is_none());
}

#[test]
fn test_drop_empty() {
    let mut buffer = crate::Dynamic::<f32>::new();

    assert_eq!(buffer.frames(), 0);
    buffer.resize(1024);
    assert_eq!(buffer.frames(), 1024);
}

#[test]
fn test_into_vecs() {
    let mut buffer = crate::Dynamic::<f32>::new();
    buffer.resize_channels(4);
    buffer.resize(512);

    let expected = vec![0.0; 512];

    let buffers = buffer.into_vectors();
    assert_eq!(buffers.len(), 4);
    assert_eq!(buffers[0], &expected[..]);
    assert_eq!(buffers[1], &expected[..]);
    assert_eq!(buffers[2], &expected[..]);
    assert_eq!(buffers[3], &expected[..]);
}

#[test]
fn test_enabled_mut() {
    use bittle::Mask as _;

    let mut buffer = crate::Dynamic::<f32>::with_topology(4, 1024);
    let mask: bittle::BitSet<u128> = bittle::bit_set![0, 2, 3];

    for mut chan in mask.join(buffer.iter_mut()) {
        for b in chan.iter_mut() {
            *b = 1.0;
        }
    }

    let zeroed = vec![0.0f32; 1024];
    let expected = vec![1.0f32; 1024];

    assert_eq!(&buffer[0], &expected[..]);
    assert_eq!(&buffer[1], &zeroed[..]);
    assert_eq!(&buffer[2], &expected[..]);
    assert_eq!(&buffer[3], &expected[..]);
}

#[test]
fn test_stale_allocation() {
    let mut buffer = crate::Dynamic::<f32>::with_topology(4, 256);
    assert_eq!(buffer[1][128], 0.0);
    buffer[1][128] = 42.0;

    buffer.resize(64);
    assert!(buffer[1].get(128).is_none());

    buffer.resize(256);
    assert_eq!(buffer[1][128], 42.0);
}

#[test]
fn test_from_array() {
    let _ = crate::dynamic![[0.0; 1024]; 2];
}

#[test]
fn test_get_mut() {
    use rand::Rng as _;

    let mut buffer = crate::Dynamic::<f32>::new();

    buffer.resize_channels(2);
    buffer.resize(256);

    let mut rng = rand::thread_rng();

    if let Some(mut left) = buffer.get_mut(0) {
        rng.fill(left.as_mut());
    }

    if let Some(mut right) = buffer.get_mut(1) {
        rng.fill(right.as_mut());
    }
}

#[test]
fn test_get_or_default() {
    let mut buffer = crate::Dynamic::<f32>::new();

    buffer.resize(256);

    let expected = vec![0f32; 256];

    assert_eq!(buffer.get_or_default(0), &expected[..]);
    assert_eq!(buffer.get_or_default(1), &expected[..]);

    assert_eq!(buffer.channels(), 2);
}

#[test]
fn test_resize_topology() {
    let mut buffer = crate::Dynamic::<f64>::with_topology(1, 1024);

    buffer.resize(20480);
    buffer.resize_channels(1);
}
