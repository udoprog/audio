use crate::translate::Translate;

macro_rules! test_int_mids {
    ($ty:ty, $mid:expr) => {
        assert_eq!(<$ty>::translate(32768u16), $mid);
        assert_eq!(<$ty>::translate(0i16), $mid);
        assert_eq!(<$ty>::translate(0.0f32), $mid);
        assert_eq!(<$ty>::translate(0.0f64), $mid);
    };
}

macro_rules! test_int {
    ($ty:ident, $min:expr, $max:expr) => {
        assert_eq!(<$ty>::translate(1.0f32), $max);
        assert_eq!(<$ty>::translate(-1.0f32), $min);

        assert_eq!(<$ty>::translate(1.0f64), $max);
        assert_eq!(<$ty>::translate(-1.0f64), $min);

        assert_eq!(<$ty>::translate(u16::MIN), $min);
        assert_eq!(<$ty>::translate(u16::MAX), $max);

        assert_eq!(<$ty>::translate(i16::MIN), $min);
        assert_eq!(<$ty>::translate(i16::MAX), $max);
    };
}

#[test]
fn test_u16_translations() {
    test_int_mids!(u16, 32768);
    test_int!(u16, u16::MIN, u16::MAX);
}

#[test]
fn test_i16_translations() {
    test_int_mids!(i16, 0);
    test_int!(i16, i16::MIN, i16::MAX);
}

macro_rules! test_float {
    ($ty:ident, $min:expr, $max:expr, $int_max:expr) => {
        assert_eq!(<$ty>::translate(1.0f32), $max);
        assert_eq!(<$ty>::translate(-1.0f32), $min);

        assert_eq!(<$ty>::translate(1.0f64), $max);
        assert_eq!(<$ty>::translate(-1.0f64), $min);

        assert_eq!(<$ty>::translate(u16::MIN), $min);
        assert_eq!(<$ty>::translate(u16::MAX), $int_max);
        assert_eq!(<u16>::translate($int_max), u16::MAX);

        assert_eq!(<$ty>::translate(i16::MIN), $min);
        assert_eq!(<$ty>::translate(i16::MAX), $int_max);
        assert_eq!(<i16>::translate($int_max), i16::MAX);
    };
}

#[test]
fn test_f32_translations() {
    // NB: integer max to float translations are not expected to cover the whole
    // range.
    // See: https://github.com/udoprog/audio/issues/7
    test_float!(f32, -1.0, 1.0, 0.9999695);
}

#[test]
fn test_f64_translations() {
    // NB: integer max to float translations are not expected to cover the whole
    // range.
    // See: https://github.com/udoprog/audio/issues/7
    test_float!(f64, -1.0, 1.0, 0.999969482421875);
}
