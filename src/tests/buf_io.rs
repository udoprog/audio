#[test]
fn test_read_write() {
    use crate::io::{Read, ReadWrite, Write};
    use crate::{Buf as _, ReadBuf as _, WriteBuf as _};

    let from = crate::interleaved![[1.0f32, 2.0f32, 3.0f32, 4.0f32]; 2];
    let to = crate::interleaved![[0.0f32; 4]; 2];

    // Make `to` into a ReadWrite adapter.
    let mut to = ReadWrite::new(to);

    to.copy(Read::new((&from).skip(2).limit(1)));
    assert_eq!(to.remaining(), 1);

    to.copy(Read::new((&from).limit(1)));
    assert_eq!(to.remaining(), 2);

    assert_eq! {
        to.as_ref().as_slice(),
        &[3.0, 3.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0],
    };

    // Note: 4 channels, 2 frames each.
    let mut read_out = Write::new(crate::Interleaved::with_topology(4, 2));

    assert_eq!(read_out.remaining_mut(), 2);
    assert!(read_out.has_remaining_mut());

    assert_eq!(to.remaining(), 2);
    assert!(to.has_remaining());

    read_out.copy(&mut to);

    assert_eq!(read_out.remaining_mut(), 0);
    assert!(!read_out.has_remaining_mut());

    assert_eq!(to.remaining(), 0);
    assert!(!to.has_remaining());

    assert_eq! {
        read_out.as_ref().as_slice(),
        &[3.0, 3.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0],
    }
}
