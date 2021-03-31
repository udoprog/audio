/// Construct an audio buffer.
///
/// This is useful when testing.
///
/// # Examples
///
/// ```rust
/// let buf = rotary::dynamic![[0.0; 64]; 2];
///
/// let mut expected = vec![0.0; 64];
///
/// assert_eq!(&buf[0], &expected[..]);
/// assert_eq!(&buf[1], &expected[..]);
/// ```
///
/// Calling the macro with a template channel.
///
/// ```rust
/// let buf = rotary::dynamic![[0.0, 1.0, 2.0, 3.0]; 2];
///
/// assert_eq!(buf[0].as_slice(), &[0.0, 1.0, 2.0, 3.0]);
/// assert_eq!(buf[1].as_slice(), &[0.0, 1.0, 2.0, 3.0]);
/// ```
#[macro_export]
macro_rules! dynamic {
    // Branch of the macro used when we can perform a literal instantiation of
    // the audio buffer.
    //
    // This is typically more performant, since it doesn't require looping and
    // writing through the buffer.
    ([$sample:expr; $frames:literal]; $channels:literal) => {
        $crate::Dynamic::from_array([[$sample; $frames]; $channels]);
    };

    // Branch of the macro used when we can evaluate an expression that is
    // built into an audio buffer.
    //
    // `$sample`, `$frames`, and `$channels` are all expected to implement
    // `Copy`. `$frames` and `$channels` should evaluate to `usize`.
    ([$sample:expr; $frames:expr]; $channels:expr) => {{
        let value = $sample;
        let mut buffer = $crate::Dynamic::with_topology($channels, $frames);

        for chan in &mut buffer {
            for f in chan {
                *f = value;
            }
        }

        buffer
    }};

    // Build a dynamic audio buffer with a template channel.
    ([$($value:expr),* $(,)?]; $channels:expr) => {
        $crate::Dynamic::from_frames([$($value),*], $channels)
    };
}

/// Construct a sequential audio buffer.
///
/// This is useful for testing.
///
/// # Examples
///
/// ```rust
/// let buf = rotary::sequential![[0.0; 64]; 2];
///
/// let mut expected = vec![0.0; 64];
///
/// assert_eq!(&buf[0], &expected[..]);
/// assert_eq!(&buf[1], &expected[..]);
/// ```
///
/// Calling the macro with a template channel.
///
/// ```rust
/// let buf = rotary::sequential![[0.0, 1.0, 2.0, 3.0]; 2];
///
/// assert_eq!(buf.as_slice(), &[0.0, 1.0, 2.0, 3.0, 0.0, 1.0, 2.0, 3.0])
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

    // Build a sequential audio buffer with a template channel.
    ([$($value:expr),* $(,)?]; $channels:expr) => {
        $crate::Sequential::from_frames([$($value),*], $channels)
    };
}

/// Construct an interleaved audio buffer.
///
/// This is useful for testing.
///
/// # Examples
///
/// ```rust
/// let buf = rotary::interleaved![[0.0; 64]; 2];
///
/// let mut expected = vec![0.0; 64];
///
/// assert!(buf.get(0).unwrap().iter().eq(&expected[..]));
/// assert!(buf.get(1).unwrap().iter().eq(&expected[..]));
/// ```
///
/// Calling the macro with a template channel.
///
/// ```rust
/// let buf = rotary::interleaved![[0.0, 1.0, 2.0, 3.0]; 2];
///
/// assert_eq!(buf.as_slice(), &[0.0, 0.0, 1.0, 1.0, 2.0, 2.0, 3.0, 3.0])
/// ```
#[macro_export]
macro_rules! interleaved {
    // Branch of the macro used when we can evaluate an expression that is
    // built into a interleaved audio buffer.
    //
    // `$sample`, `$frames`, and `$channels` are all expected to implement
    // `Copy`. `$frames` and `$channels` should evaluate to `usize`.
    ([$sample:expr; $frames:expr]; $channels:expr) => {
        $crate::Interleaved::from_vec(vec![$sample; $channels * $frames], $channels, $frames)
    };

    // Build an interleaved audio buffer with a template channel.
    ([$($value:expr),* $(,)?]; $channels:expr) => {
        $crate::Interleaved::from_frames([$($value),*], $channels)
    };
}
