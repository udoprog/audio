#[test]
fn test_read_write() {
    use crate::io::{Read, ReadWrite, Write};
    use audio_core::{Buf, ReadBuf, WriteBuf};

    let from = crate::interleaved![[1.0f32, 2.0f32, 3.0f32, 4.0f32]; 2];
    let to = crate::interleaved![[0.0f32; 4]; 2];

    // Make `to` into a ReadWrite adapter.
    let mut to = ReadWrite::empty(to);

    crate::io::copy_remaining(Read::new((&from).skip(2).limit(1)), &mut to);
    assert_eq!(to.remaining(), 1);

    crate::io::copy_remaining(Read::new((&from).limit(1)), &mut to);
    assert_eq!(to.remaining(), 2);

    assert_eq! {
        to.as_ref().as_slice(),
        &[3.0, 3.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0],
    };

    // Note: 4 channels, 2 frames each.
    let mut read_out = Write::new(crate::buf::Interleaved::with_topology(4, 2));

    assert_eq!(read_out.remaining_mut(), 2);
    assert!(read_out.has_remaining_mut());

    assert_eq!(to.remaining(), 2);
    assert!(to.has_remaining());

    crate::io::copy_remaining(&mut to, &mut read_out);

    assert_eq!(read_out.remaining_mut(), 0);
    assert!(!read_out.has_remaining_mut());

    assert_eq!(to.remaining(), 0);
    assert!(!to.has_remaining());

    assert_eq! {
        read_out.as_ref().as_slice(),
        &[3.0, 3.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0],
    }
}

#[test]
fn test_simple_io() {
    use crate::io::ReadWrite;

    let buf: crate::buf::Interleaved<i16> = crate::interleaved![[1, 2, 3, 4]; 4];
    let mut buf = ReadWrite::new(buf);

    let from = crate::wrap::interleaved(&[1i16, 2i16, 3i16, 4i16][..], 2);

    crate::io::translate_remaining(from, &mut buf);

    let buf = buf.into_inner();

    assert_eq!(buf.channels(), 4);
}
