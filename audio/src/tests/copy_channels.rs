// Miri: copying channels internally in a buffer intrinsically requires a bit of
// tongue in cheek pointer mangling. These tests are added here so that they can
// be run through miri to test that at least a base level of sanity is
// maintained.

#[test]
fn test_copy_channels_dynamic() {
    use crate::{Channels, ChannelsMut};

    let mut buffer: crate::Dynamic<i16> = crate::dynamic![[1, 2, 3, 4], [0, 0, 0, 0]];
    buffer.copy_channels(0, 1);

    assert_eq!(buffer.channel(1), buffer.channel(0));
}

#[test]
fn test_copy_channels_sequential() {
    use crate::{Channels, ChannelsMut};

    let mut buffer: crate::Sequential<i16> = crate::sequential![[1, 2, 3, 4], [0, 0, 0, 0]];
    buffer.copy_channels(0, 1);

    assert_eq!(buffer.channel(1), buffer.channel(0));
    assert_eq!(buffer.as_slice(), &[1, 2, 3, 4, 1, 2, 3, 4]);
}

#[test]
fn test_copy_channels_wrap_sequential() {
    use crate::wrap;
    use crate::{Channels, ChannelsMut};

    let mut data = [1, 2, 3, 4, 0, 0, 0, 0];
    let data = &mut data[..];
    let mut buffer: wrap::Sequential<&mut [i16]> = wrap::sequential(data, 2);
    buffer.copy_channels(0, 1);

    assert_eq!(buffer.channel(1), buffer.channel(0));
    assert_eq!(data, &[1, 2, 3, 4, 1, 2, 3, 4]);
}

#[test]
fn test_copy_channels_interleaved() {
    use crate::{Channels, ChannelsMut};

    let mut buffer: crate::Interleaved<i16> = crate::interleaved![[1, 2, 3, 4], [0, 0, 0, 0]];
    buffer.copy_channels(0, 1);

    assert_eq!(buffer.channel(1), buffer.channel(0));
    assert_eq!(buffer.as_slice(), &[1, 1, 2, 2, 3, 3, 4, 4]);
}

#[test]
fn test_copy_channels_wrap_interleaved() {
    use crate::wrap;
    use crate::{Channels, ChannelsMut};

    let mut data = [1, 0, 2, 0, 3, 0, 4, 0];
    let mut buffer: wrap::Interleaved<&mut [i16]> = wrap::interleaved(&mut data[..], 2);
    buffer.copy_channels(0, 1);

    assert_eq!(buffer.channel(1), buffer.channel(0));
    assert_eq!(&data[..], &[1, 1, 2, 2, 3, 3, 4, 4]);
}

#[test]
fn test_copy_channels_vec_of_vecs() {
    use crate::{Channels, ChannelsMut};

    let mut buffer: Vec<Vec<i16>> = vec![vec![1, 2, 3, 4], vec![0, 0]];
    buffer.copy_channels(0, 1);

    assert_eq!(buffer.channel(1), buffer.channel(0).limit(2));
}
