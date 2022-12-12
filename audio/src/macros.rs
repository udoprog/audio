/// Construct an audio buffer.
///
/// This is useful when testing.
///
/// # Examples
///
/// ```
/// let buf = audio::dynamic![[0; 64]; 2];
///
/// let mut expected = vec![0; 64];
///
/// assert_eq!(&buf[0], &expected[..]);
/// assert_eq!(&buf[1], &expected[..]);
/// ```
///
/// Calling the macro with a template channel.
///
/// ```
/// let buf = audio::dynamic![[0, 1, 2, 3]; 2];
///
/// assert_eq!(&buf[0], &[0, 1, 2, 3]);
/// assert_eq!(&buf[1], &[0, 1, 2, 3]);
/// ```
///
/// Using an exact topology of channels.
///
/// ```
/// let buf = audio::dynamic![[0, 1, 2, 3], [4, 5, 6, 7]];
///
/// assert_eq!(&buf[0], &[0, 1, 2, 3]);
/// assert_eq!(&buf[1], &[4, 5, 6, 7]);
/// ```
#[cfg(feature = "std")]
#[macro_export]
macro_rules! dynamic {
    // Branch of the macro used when we can perform a literal instantiation of
    // the audio buffer.
    //
    // This is typically more performant, since it doesn't require looping and
    // writing through the buffer.
    ([$sample:expr; $frames:literal]; $channels:literal) => {
        $crate::buf::Dynamic::from_array([[$sample; $frames]; $channels])
    };

    // Branch of the macro used when we can evaluate an expression that is
    // built into an audio buffer.
    //
    // `$sample`, `$frames`, and `$channels` are all expected to implement
    // `Copy`. `$frames` and `$channels` should evaluate to `usize`.
    ([$sample:expr; $frames:expr]; $channels:expr) => {{
        let value = $sample;
        let mut buf = $crate::buf::Dynamic::with_topology($channels, $frames);

        for mut chan in buf.iter_mut() {
            for f in chan.iter_mut() {
                *f = value;
            }
        }

        buf
    }};

    // Build a dynamic audio buffer with a template channel.
    ([$($value:expr),* $(,)?]; $channels:expr) => {
        $crate::buf::Dynamic::from_frames([$($value),*], $channels)
    };

    // Build a dynamic audio buffer from a specific topology of channels.
    ($($channel:expr),* $(,)?) => {
        $crate::buf::Dynamic::from_array([$($channel),*])
    };
}

/// Construct a sequential audio buffer.
///
/// This is useful for testing.
///
/// # Examples
///
/// ```
/// let buf = audio::sequential![[0; 64]; 2];
///
/// let mut expected = vec![0; 64];
///
/// assert_eq!(&buf[0], &expected[..]);
/// assert_eq!(&buf[1], &expected[..]);
/// ```
///
/// Calling the macro with a template channel.
///
/// ```
/// let buf = audio::sequential![[0, 1, 2, 3]; 2];
///
/// assert_eq!(buf.as_slice(), &[0, 1, 2, 3, 0, 1, 2, 3])
/// ```
///
/// Using an exact topology of channels.
///
/// ```
/// let buf = audio::sequential![[0, 1, 2, 3], [4, 5, 6, 7]];
///
/// assert_eq!(buf.as_slice(), &[0, 1, 2, 3, 4, 5, 6, 7])
/// ```
#[cfg(feature = "std")]
#[macro_export]
macro_rules! sequential {
    // Branch of the macro used when we can evaluate an expression that is
    // built into a sequential audio buffer.
    //
    // `$sample`, `$frames`, and `$channels` are all expected to implement
    // `Copy`. `$frames` and `$channels` should evaluate to `usize`.
    ([$sample:expr; $frames:expr]; $channels:expr) => {
        $crate::buf::Sequential::from_vec(vec![$sample; $channels * $frames], $channels, $frames)
    };

    // Build a sequential audio buffer with a template channel.
    ([$($value:expr),* $(,)?]; $channels:expr) => {
        $crate::buf::Sequential::from_frames([$($value),*], $channels)
    };

    // Build a sequential audio buffer from a specific topology of channels.
    ($($channel:expr),* $(,)?) => {
        $crate::buf::Sequential::from_array([$($channel),*])
    };
}

/// Construct an interleaved audio buffer.
///
/// This is useful for testing.
///
/// # Examples
///
/// ```
/// let buf = audio::interleaved![[0; 64]; 2];
///
/// let mut expected = vec![0; 64];
///
/// assert!(buf.get_channel(0).unwrap().iter().eq(expected.iter().copied()));
/// assert!(buf.get_channel(1).unwrap().iter().eq(expected.iter().copied()));
/// ```
///
/// Calling the macro with a template channel.
///
/// ```
/// let buf = audio::interleaved![[0, 1, 2, 3]; 2];
///
/// assert_eq!(buf.as_slice(), &[0, 0, 1, 1, 2, 2, 3, 3])
/// ```
///
/// Using an exact topology of channels.
///
/// ```
/// let buf = audio::interleaved![[0, 1, 2, 3], [4, 5, 6, 7]];
///
/// assert_eq!(buf.as_slice(), &[0, 4, 1, 5, 2, 6, 3, 7])
/// ```
#[cfg(feature = "std")]
#[macro_export]
macro_rules! interleaved {
    // Branch of the macro used when we can evaluate an expression that is
    // built into a interleaved audio buffer.
    //
    // `$sample`, `$frames`, and `$channels` are all expected to implement
    // `Copy`. `$frames` and `$channels` should evaluate to `usize`.
    ([$sample:expr; $frames:expr]; $channels:expr) => {
        $crate::buf::Interleaved::from_vec(vec![$sample; $channels * $frames], $channels, $frames)
    };

    // Build an interleaved audio buffer with a template channel.
    ([$($value:expr),* $(,)?]; $channels:expr) => {
        $crate::buf::Interleaved::from_frames([$($value),*], $channels)
    };

    // Build an interleaved audio buffer from a specific topology of channels.
    ($($channel:expr),* $(,)?) => {
        $crate::buf::Interleaved::from_array([$($channel),*])
    };
}
