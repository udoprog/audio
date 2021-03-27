/// Instantiate a literal audio buffer.
///
/// This is useful when testing.
///
/// # Examples
///
/// ```rust
/// let buf = rotary::audio_buffer![[0.0; 1024]; 2];
///
/// let mut expected = vec![0.0; 1024];
///
/// assert_eq!(&buf[0], &expected[..]);
/// assert_eq!(&buf[1], &expected[..]);
/// ```
#[macro_export]
macro_rules! audio_buffer {
    ([$inst:expr; $frames:expr]; $channels:expr) => {
        $crate::AudioBuffer::from([[$inst; $frames]; $channels]);
    };
}
