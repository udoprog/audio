/// Note: most of these tests are duplicated doc tests, but they're here so that
/// we can run them through miri and get a good idea of the soundness of our
/// implementations.

#[test]
fn test_channels_then_resize() {
    let mut buffer = crate::Dynamic::<f32>::new();

    buffer.resize_channels(4);
    buffer.resize(1024);

    let expected = vec![0.0; 1024];

    assert_eq!(Some(&expected[..]), buffer.get(0));
    assert_eq!(Some(&expected[..]), buffer.get(1));
    assert_eq!(Some(&expected[..]), buffer.get(2));
    assert_eq!(Some(&expected[..]), buffer.get(3));
    assert_eq!(None, buffer.get(4));
}

#[test]
fn test_resize_then_channels() {
    let mut buffer = crate::Dynamic::<f32>::new();

    buffer.resize(1024);
    buffer.resize_channels(4);

    let expected = vec![0.0; 1024];

    assert_eq!(buffer.get(0), Some(&expected[..]));
    assert_eq!(buffer.get(1), Some(&expected[..]));
    assert_eq!(buffer.get(2), Some(&expected[..]));
    assert_eq!(buffer.get(3), Some(&expected[..]));
    assert_eq!(buffer.get(4), None);
}

#[test]
fn test_empty_channels() {
    let mut buffer = crate::Dynamic::<f32>::new();

    buffer.resize_channels(4);

    assert_eq!(buffer.get(0), Some(&[][..]));
    assert_eq!(buffer.get(1), Some(&[][..]));
    assert_eq!(buffer.get(2), Some(&[][..]));
    assert_eq!(buffer.get(3), Some(&[][..]));
    assert_eq!(buffer.get(4), None);
}

#[test]
fn test_empty() {
    let buffer = crate::Dynamic::<f32>::new();

    assert_eq!(buffer.frames(), 0);
    assert_eq!(buffer.get(0), None);
}

#[test]
fn test_multiple_resizes() {
    let mut buffer = crate::Dynamic::<f32>::new();

    buffer.resize_channels(4);
    buffer.resize(1024);

    let expected = vec![0.0; 1024];

    assert_eq!(buffer.get(0), Some(&expected[..]));
    assert_eq!(buffer.get(1), Some(&expected[..]));
    assert_eq!(buffer.get(2), Some(&expected[..]));
    assert_eq!(buffer.get(3), Some(&expected[..]));
    assert_eq!(buffer.get(4), None);
}

#[test]
fn test_multiple_channel_resizes() {
    let mut buffer = crate::Dynamic::<f32>::new();

    buffer.resize_channels(4);
    buffer.resize(1024);

    let expected = vec![0.0; 1024];

    assert_eq!(buffer.get(0), Some(&expected[..]));
    assert_eq!(buffer.get(1), Some(&expected[..]));
    assert_eq!(buffer.get(2), Some(&expected[..]));
    assert_eq!(buffer.get(3), Some(&expected[..]));
    assert_eq!(buffer.get(4), None);

    buffer.resize_channels(2);

    assert_eq!(buffer.get(0), Some(&expected[..]));
    assert_eq!(buffer.get(1), Some(&expected[..]));
    assert_eq!(buffer.get(2), None);
    assert_eq!(buffer.get(3), None);
    assert_eq!(buffer.get(4), None);
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
    use crate::Mask as _;

    let mut buffer = crate::Dynamic::<f32>::with_topology(4, 1024);
    let mask: crate::BitSet<u128> = crate::bit_set![0, 2, 3];

    for chan in mask.join(buffer.iter_mut()) {
        for b in chan {
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

    if let Some(left) = buffer.get_mut(0) {
        rng.fill(left);
    }

    if let Some(right) = buffer.get_mut(1) {
        rng.fill(right);
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
