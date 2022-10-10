// Miri: copying channels internally in a buffer intrinsically requires a bit of
// tongue in cheek pointer mangling. These tests are added here so that they can
// be run through miri to test that at least a base level of sanity is
// maintained.

use crate::{buf, wrap};
use audio_core::{Buf, BufMut, Channel};

#[test]
fn test_copy_channels_dynamic() {
    let mut buf: buf::Dynamic<i16> = crate::dynamic![[1, 2, 3, 4], [0, 0, 0, 0]];
    buf.copy_channel(0, 1);

    assert_eq!(buf.channel(1), buf.channel(0));
}

#[test]
fn test_copy_channels_sequential() {
    let mut buf: buf::Sequential<i16> = crate::sequential![[1, 2, 3, 4], [0, 0, 0, 0]];
    buf.copy_channel(0, 1);

    assert_eq!(buf.channel(1), buf.channel(0));
    assert_eq!(buf.as_slice(), &[1, 2, 3, 4, 1, 2, 3, 4]);
}

#[test]
fn test_copy_channels_wrap_sequential() {
    let mut data = [1, 2, 3, 4, 0, 0, 0, 0];
    let data = &mut data[..];
    let mut buf: wrap::Sequential<&mut [i16]> = wrap::sequential(data, 2);
    buf.copy_channel(0, 1);

    assert_eq!(buf.channel(1), buf.channel(0));
    assert_eq!(data, &[1, 2, 3, 4, 1, 2, 3, 4]);
}

#[test]
fn test_copy_channels_interleaved() {
    let mut buf: buf::Interleaved<i16> = crate::interleaved![[1, 2, 3, 4], [0, 0, 0, 0]];
    buf.copy_channel(0, 1);

    assert_eq!(buf.channel(1), buf.channel(0));
    assert_eq!(buf.as_slice(), &[1, 1, 2, 2, 3, 3, 4, 4]);
}

#[test]
fn test_copy_channels_wrap_interleaved() {
    let mut data = [1, 0, 2, 0, 3, 0, 4, 0];
    let mut buf: wrap::Interleaved<&mut [i16]> = wrap::interleaved(&mut data[..], 2);
    buf.copy_channel(0, 1);

    assert_eq!(buf.channel(1), buf.channel(0));
    assert_eq!(&data[..], &[1, 1, 2, 2, 3, 3, 4, 4]);
}

#[test]
fn test_copy_channels_vec_of_vecs() {
    let mut buf = crate::wrap::dynamic(vec![vec![1, 2, 3, 4], vec![0, 0]]);
    buf.copy_channel(0, 1);

    assert_eq!(buf.channel(1).unwrap(), buf.channel(0).unwrap().limit(2));
}
