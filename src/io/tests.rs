#[test]
fn test_simple_io() {
    use crate::io::ReadWrite;
    use crate::WriteBuf as _;

    let buffer: crate::Interleaved<i16> = crate::interleaved![[1, 2, 3, 4]; 4];
    let mut buffer = ReadWrite::new(buffer);

    let from = crate::wrap::interleaved(&[1i16, 2i16, 3i16, 4i16][..], 2);

    buffer.translate(from);

    let buffer = buffer.into_inner();

    assert_eq!(buffer.channels(), 4);
}
