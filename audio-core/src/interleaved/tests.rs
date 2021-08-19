use super::{InterleavedChannel, InterleavedChannelMut};
use crate::{Channel, ChannelMut};
use std::ptr;

#[test]
fn test_interleaved_channel() {
    let buf: &[u32] = &[1, 2, 3, 4, 5, 6][..];

    let c1 = unsafe {
        InterleavedChannel::new_unchecked(
            ptr::NonNull::new_unchecked(buf.as_ptr() as *mut u32),
            buf.len(),
            0,
            2,
        )
    };

    let c2 = unsafe {
        InterleavedChannel::new_unchecked(
            ptr::NonNull::new_unchecked(buf.as_ptr() as *mut u32),
            buf.len(),
            1,
            2,
        )
    };

    assert_eq!(c1.iter().collect::<Vec<_>>(), vec![1, 3, 5]);
    assert_eq!(c2.iter().collect::<Vec<_>>(), vec![2, 4, 6]);
}

#[test]
fn test_interleaved_channel_mut() {
    let buf: &mut [u32] = &mut [1, 2, 3, 4, 5, 6][..];

    let c1 = unsafe {
        InterleavedChannelMut::new_unchecked(
            ptr::NonNull::new_unchecked(buf.as_mut_ptr() as *mut u32),
            buf.len(),
            0,
            2,
        )
    };

    let c2 = unsafe {
        InterleavedChannelMut::new_unchecked(
            ptr::NonNull::new_unchecked(buf.as_mut_ptr() as *mut u32),
            buf.len(),
            1,
            2,
        )
    };

    assert_eq!(c1.iter().collect::<Vec<_>>(), vec![1, 3, 5]);
    assert_eq!(c2.iter().collect::<Vec<_>>(), vec![2, 4, 6]);
}

#[test]
fn test_interleaved_channel_mut_iter() {
    let buf: &mut [u32] = &mut [1, 2, 3, 4, 5, 6][..];

    let mut c1 = unsafe {
        InterleavedChannelMut::new_unchecked(
            ptr::NonNull::new_unchecked(buf.as_mut_ptr() as *mut u32),
            buf.len(),
            0,
            2,
        )
    };

    let c2 = unsafe {
        InterleavedChannelMut::new_unchecked(
            ptr::NonNull::new_unchecked(buf.as_mut_ptr() as *mut u32),
            buf.len(),
            1,
            2,
        )
    };

    for s in c1.iter_mut() {
        *s += 2;
    }

    assert_eq!(c1.iter().collect::<Vec<_>>(), vec![3, 5, 7]);
    assert_eq!(c2.iter().collect::<Vec<_>>(), vec![2, 4, 6]);

    assert_eq!(buf, &mut [3, 2, 5, 4, 7, 6][..]);
}

macro_rules! slice_tests {
    (
        $ty:ty,
        ($ch:expr, $channels:expr) => $input:expr,
        $($fn:ident($($arg:expr),* $(,)?) => [$($expected:expr),* $(,)?]),* $(,)?
    ) => {{
        let buf = $input;

        let c = unsafe {
            InterleavedChannel::new_unchecked(
                ptr::NonNull::new_unchecked(buf.as_ptr() as *mut $ty),
                buf.len(),
                $ch,
                $channels,
            )
        };

        $(
        assert_eq!(
            c.$fn($($arg),*).iter().collect::<Vec<_>>(),
            vec![$($expected),*],
            "{}.{}({}) != {}",
            stringify!($input),
            stringify!($fn),
            stringify!($($arg),*),
            stringify!([$($expected),*]),
        );
        )*
    }}
}

#[test]
fn test_skip() {
    slice_tests! {
        u32,
        (0, 2) => [1, 2, 3, 4, 5, 6],
        skip(0) => [1, 3, 5],
        skip(1) => [3, 5],
        skip(2) => [5],
        skip(3) => [],
        skip(4) => [],
    }

    slice_tests! {
        u32,
        (1, 2) => [1, 2, 3, 4, 5, 6],
        skip(0) => [2, 4, 6],
        skip(1) => [4, 6],
        skip(2) => [6],
        skip(3) => [],
        skip(4) => [],
    }

    // ZST
    slice_tests! {
        (),
        (1, 2) => [(), (), (), (), (), ()],
        skip(0) => [(), (), ()],
        skip(1) => [(), ()],
        skip(2) => [()],
        skip(3) => [],
        skip(4) => [],
    }
}

#[test]
fn test_tail() {
    slice_tests! {
        u32,
        (0, 2) => [1, 2, 3, 4, 5, 6],
        tail(0) => [],
        tail(1) => [5],
        tail(2) => [3, 5],
        tail(3) => [1, 3, 5],
        tail(4) => [1, 3, 5],
    }

    slice_tests! {
        u32,
        (1, 2) => [1, 2, 3, 4, 5, 6],
        tail(0) => [],
        tail(1) => [6],
        tail(2) => [4, 6],
        tail(3) => [2, 4, 6],
        tail(4) => [2, 4, 6],
    }

    // ZST
    slice_tests! {
        (),
        (1, 2) => [(), (), (), (), (), ()],
        tail(0) => [],
        tail(1) => [()],
        tail(2) => [(), ()],
        tail(3) => [(), (), ()],
        tail(4) => [(), (), ()],
    }
}

#[test]
fn test_limit() {
    slice_tests! {
        u32,
        (0, 2) => [1, 2, 3, 4, 5, 6],
        limit(0) => [],
        limit(1) => [1],
        limit(2) => [1, 3],
        limit(3) => [1, 3, 5],
        limit(4) => [1, 3, 5],
    }

    slice_tests! {
        u32,
        (1, 2) => [1, 2, 3, 4, 5, 6],
        limit(0) => [],
        limit(1) => [2],
        limit(2) => [2, 4],
        limit(3) => [2, 4, 6],
        limit(4) => [2, 4, 6],
    }

    // ZST
    slice_tests! {
        (),
        (1, 2) => [(), (), (), (), (), ()],
        limit(0) => [],
        limit(1) => [()],
        limit(2) => [(), ()],
        limit(3) => [(), (), ()],
        limit(4) => [(), (), ()],
    }
}

#[test]
fn test_chunk() {
    slice_tests! {
        u32,
        (1, 2) => &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10][..],
        chunk(0, 2) => [2, 4],
        chunk(1, 2) => [6, 8],
        chunk(2, 2) => [10],
        chunk(3, 2) => [],
    }

    slice_tests! {
        u32,
        (0, 2) => &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10][..],
        chunk(0, 2) => [1, 3],
        chunk(1, 2) => [5, 7],
        chunk(2, 2) => [9],
        chunk(3, 2) => [],
    }

    // ZST
    slice_tests! {
        (),
        (0, 2) => &[(), (), (), (), (), (), (), (), (), ()][..],
        chunk(0, 2) => [(), ()],
        chunk(1, 2) => [(), ()],
        chunk(2, 2) => [()],
        chunk(3, 2) => [],
    }
}

#[test]
fn test_interleaved_channel_count() {
    macro_rules! test {
        (
            $ty:ty,
            ($ch:expr, $channels:expr) => $input:expr,
            $expected:expr $(,)?
        ) => {{
            let buf = $input;

            let c = unsafe {
                InterleavedChannel::new_unchecked(
                    ptr::NonNull::new_unchecked(buf.as_ptr() as *mut $ty),
                    buf.len(),
                    $ch,
                    $channels,
                )
            };

            let mut it = c.iter();
            assert_eq!(it.next(), $expected);
            assert_eq!(it.count(), 2);
        }};
    }

    test! {
        u32,
        (0, 2) => &[1, 2, 3, 4, 5, 6][..],
        Some(1),
    }

    test! {
        u32,
        (1, 2) => &[1, 2, 3, 4, 5, 6][..],
        Some(2),
    }

    test! {
        (),
        (1, 2) => &[(), (), (), (), (), ()][..],
        Some(()),
    }
}

#[test]
fn test_interleaved_channel_nth() {
    macro_rules! test {
        (
            $ty:ty,
            ($ch:expr, $channels:expr) => $input:expr,
            $($a:expr => $expected:expr),* $(,)?
        ) => {{
            let buf = $input;

            let c = unsafe {
                InterleavedChannel::new_unchecked(
                    ptr::NonNull::new_unchecked(buf.as_ptr() as *mut $ty),
                    buf.len(),
                    $ch,
                    $channels,
                )
            };

            $(
                assert_eq!(c.iter().nth($a), $expected);
            )*
        }}
    }

    test! {
        u32,
        (0, 2) => &[1, 2, 3, 4, 5, 6][..],
        0 => Some(1),
        1 => Some(3),
        2 => Some(5),
        3 => None,
        4 => None,
    }

    test! {
        u32,
        (1, 2) => &[1, 2, 3, 4, 5, 6][..],
        0 => Some(2),
        1 => Some(4),
        2 => Some(6),
        3 => None,
        4 => None,
    }

    // ZST
    test! {
        (),
        (1, 2) => &[(), (), (), (), (), ()][..],
        0 => Some(()),
        1 => Some(()),
        2 => Some(()),
        3 => None,
        4 => None,
    }
}

#[test]
fn test_interleaved_channel_next_back() {
    let buf: &[u32] = &[1, 2, 3, 4, 5, 6][..];

    let c1 = unsafe {
        InterleavedChannel::new_unchecked(
            ptr::NonNull::new_unchecked(buf.as_ptr() as *mut u32),
            buf.len(),
            1,
            2,
        )
    };

    let mut it = c1.iter();
    assert_eq!(it.next_back(), Some(6));
    assert_eq!(it.next(), Some(2));
    assert_eq!(it.next_back(), Some(4));
    assert_eq!(it.next(), None);
}
