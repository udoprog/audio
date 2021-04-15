use crate::translate::Translate;

macro_rules! test {
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
    test!(u16, u16::MIN, u16::MAX);
}

#[test]
fn test_i16_translations() {
    test!(i16, i16::MIN, i16::MAX);
}

#[test]
fn test_f32_translations() {
    test!(f32, -1.0, 1.0);
}

#[test]
fn test_f64_translations() {
    test!(f64, -1.0, 1.0);
}
