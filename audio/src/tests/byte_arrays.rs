#[test]
fn test_byte_array() {
    let buf: crate::buf::Interleaved<[u8; 2]> =
        crate::interleaved![[[1, 2], [3, 4]], [[5, 6], [7, 8]]];

    assert_eq!(buf.channels(), 2);
    assert_eq!(buf.sample(0, 0).unwrap(), [1, 2]);
    assert_eq!(buf.sample(0, 1).unwrap(), [3, 4]);
    assert_eq!(buf.sample(1, 0).unwrap(), [5, 6]);
    assert_eq!(buf.sample(1, 1).unwrap(), [7, 8]);
}
