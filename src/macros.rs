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
    // Branch of the macro used when we can perform a literal instantiation of
    // the audio buffer.
    //
    // This is typically more performant, since it doesn't require looping and
    // writing through the buffer.
    ([$sample:expr; $frames:literal]; $channels:literal) => {
        $crate::AudioBuffer::from_array([[$sample; $frames]; $channels]);
    };

    // Branch of the macro used when we can evaluate an expression that is
    // built into an audio buffer.
    //
    // `$sample`, `$frames`, and `$channels` are all expected to implement
    // `Copy`. `$frames` and `$channels` should evaluate to `usize`.
    ([$sample:expr; $frames:expr]; $channels:expr) => {{
        let value = $sample;
        let mut buffer = $crate::AudioBuffer::with_topology($channels, $frames);

        for chan in &mut buffer {
            for f in chan {
                *f = value;
            }
        }

        buffer
    }};
}

/// Instantiate a literal sequential audio buffer.
///
/// This is useful for testing.
///
/// # Examples
///
/// ```rust
/// let buf = rotary::sequential![[0.0; 1024]; 2];
///
/// let mut expected = vec![0.0; 1024];
///
/// assert_eq!(&buf[0], &expected[..]);
/// assert_eq!(&buf[1], &expected[..]);
/// ```
#[macro_export]
macro_rules! sequential {
    // Branch of the macro used when we can evaluate an expression that is
    // built into a sequential audio buffer.
    //
    // `$sample`, `$frames`, and `$channels` are all expected to implement
    // `Copy`. `$frames` and `$channels` should evaluate to `usize`.
    ([$sample:expr; $frames:expr]; $channels:expr) => {
        $crate::Sequential::from_vec(vec![$sample; $channels * $frames], $channels, $frames)
    };
}
