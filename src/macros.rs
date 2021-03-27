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
    ([$inst:expr; $frames:literal]; $channels:literal) => {
        $crate::AudioBuffer::from_array([[$inst; $frames]; $channels]);
    };

    ([$inst:expr; $frames:expr]; $channels:expr) => {{
        let value = $inst;
        let mut buffer = $crate::AudioBuffer::with_topology($channels, $frames);

        for chan in &mut buffer {
            for f in chan {
                *f = value;
            }
        }

        buffer
    }};
}
