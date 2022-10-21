use crate::translate::{IntTranslationError, Translate, TryTranslate};

macro_rules! assert_mid_ok {
    ($ty:ty, $mid:expr) => {
        let mid: $ty = $mid;
        assert_eq!(<$ty>::try_translate(0.0f32).unwrap(), mid);
        assert_eq!(<$ty>::try_translate(0.0f64).unwrap(), mid);
        assert_eq!(<$ty>::try_translate(0x80u8).unwrap(), mid);
        assert_eq!(<$ty>::try_translate(0i8).unwrap(), mid);
        assert_eq!(<$ty>::try_translate(0x8000u16).unwrap(), mid);
        assert_eq!(<$ty>::try_translate(0i16).unwrap(), mid);
        assert_eq!(<$ty>::try_translate(0x80000000u32).unwrap(), mid);
        assert_eq!(<$ty>::try_translate(0i32).unwrap(), mid);
        assert_eq!(<$ty>::try_translate(0x8000000000000000u64).unwrap(), mid);
        assert_eq!(<$ty>::try_translate(0i64).unwrap(), mid);
    };
}

#[test]
fn test_translation_mids() {
    assert_mid_ok!(u8, 0x80);
    assert_mid_ok!(u16, 0x8000);
    assert_mid_ok!(u32, 0x80000000);
    assert_mid_ok!(u64, 0x8000000000000000);
    assert_mid_ok!(i8, 0);
    assert_mid_ok!(i16, 0);
    assert_mid_ok!(i32, 0);
    assert_mid_ok!(i64, 0);
    assert_mid_ok!(f32, 0.0);
    assert_mid_ok!(f64, 0.0);
}

macro_rules! assert_min_ok {
    ($ty:ty, $min:expr) => {
        let min: $ty = $min;
        assert_eq!(<$ty>::try_translate(-1.0f32).unwrap(), min);
        assert_eq!(<$ty>::try_translate(-1.0f64).unwrap(), min);
        assert_eq!(<$ty>::try_translate(0u8).unwrap(), min);
        assert_eq!(<$ty>::try_translate(i8::MIN).unwrap(), min);
        assert_eq!(<$ty>::try_translate(0u16).unwrap(), min);
        assert_eq!(<$ty>::try_translate(i16::MIN).unwrap(), min);
        assert_eq!(<$ty>::try_translate(0u32).unwrap(), min);
        assert_eq!(<$ty>::try_translate(i32::MIN).unwrap(), min);
        assert_eq!(<$ty>::try_translate(0u64).unwrap(), min);
        assert_eq!(<$ty>::try_translate(i64::MIN).unwrap(), min);
    };
}

#[test]
fn test_translation_minimums() {
    assert_min_ok!(u8, 0);
    assert_min_ok!(u16, 0);
    assert_min_ok!(u32, 0);
    assert_min_ok!(u64, 0);
    assert_min_ok!(i8, i8::MIN);
    assert_min_ok!(i16, i16::MIN);
    assert_min_ok!(i32, i32::MIN);
    assert_min_ok!(i64, i64::MIN);
    assert_min_ok!(f32, -1.0);
    assert_min_ok!(f64, -1.0);
}

macro_rules! assert_int_max {
    ($ty:ident, $max:expr, $other_ty:ty, $other_max:expr) => {
        assert_eq!(<$ty>::try_translate($other_max), Ok($max));
        assert_eq!(<$other_ty>::try_translate($max), Ok($other_max));
    };
}

macro_rules! assert_float_max {
    ($ty:ident, $u8_max:expr, $u16_max:expr, $u32_max:expr, $u64_max:expr) => {
        let max: $ty = 1.0;
        assert_eq!(<$ty>::translate(1.0f32), max);
        assert_eq!(<$ty>::translate(1.0f64), max);

        assert_eq!(<$ty>::translate(u8::MAX), $u8_max);
        assert_eq!(<u8>::translate($u8_max), u8::MAX);
        assert_eq!(<$ty>::translate(i8::MAX), $u8_max);
        assert_eq!(<i8>::translate($u8_max), i8::MAX);

        assert_eq!(<$ty>::translate(u16::MAX), $u16_max);
        assert_eq!(<u16>::translate($u16_max), u16::MAX);
        assert_eq!(<$ty>::translate(i16::MAX), $u16_max);
        assert_eq!(<i16>::translate($u16_max), i16::MAX);

        assert_eq!(<$ty>::translate(u32::MAX), $u32_max);
        assert_eq!(<u32>::translate($u32_max), u32::MAX);
        assert_eq!(<$ty>::translate(i32::MAX), $u32_max);
        assert_eq!(<i32>::translate($u32_max), i32::MAX);

        assert_eq!(<$ty>::translate(u64::MAX), $u64_max);
        assert_eq!(<u64>::translate($u64_max), u64::MAX);
        assert_eq!(<$ty>::translate(i64::MAX), $u64_max);
        assert_eq!(<i64>::translate($u64_max), i64::MAX);
    };
}

#[test]
fn test_maximums() {
    // NB: inprecise integer translations are not expected to cover the whole
    // range of a type.
    // See: https://github.com/udoprog/audio/issues/7

    assert_float_max!(f32, 0.9921875, 0.9999695, 1.0, 1.0);
    assert_float_max!(f64, 0.9921875, 0.999969482421875, 0.9999999995343387, 1.0);

    assert_int_max!(u8, u8::MAX, u16, (u8::MAX as u16) << 8);
    assert_int_max!(u8, u8::MAX, u32, (u8::MAX as u32) << 24);
    assert_int_max!(u8, u8::MAX, u64, (u8::MAX as u64) << 56);
    assert_int_max!(u16, u16::MAX, u32, (u16::MAX as u32) << 16);
    assert_int_max!(u16, u16::MAX, u64, (u16::MAX as u64) << 48);
    assert_int_max!(u32, u32::MAX, u64, (u32::MAX as u64) << 32);
}

macro_rules! assert_ok {
    ($in_ty:ty, $in:expr, $out_ty:ty, $out:expr) => {
        let o: $out_ty = $out;
        let i: $in_ty = $in;
        assert_eq!(<$out_ty>::translate(i), o);
        assert_eq!(<$in_ty>::try_translate(o).unwrap(), i);
    };
}

#[test]
fn test_lossless_unsigned() {
    assert_ok!(u8, 0, u16, 0);
    assert_ok!(u8, u8::MAX, u16, (u8::MAX as u16) << 8);
    assert_ok!(u8, u8::MIN, u16, u16::MIN);

    assert_ok!(u16, 0, u16, 0);
    assert_ok!(u16, u16::MAX, u16, u16::MAX);
    assert_ok!(u16, u16::MIN, u16, u16::MIN);

    assert_ok!(u8, 0, u32, 0);
    assert_ok!(u8, u8::MAX, u32, (u8::MAX as u32) << 24);
    assert_ok!(u8, u8::MIN, u32, u32::MIN);

    assert_ok!(u16, 0u16, u32, 0u32);
    assert_ok!(u16, u16::MAX, u32, (u16::MAX as u32) << 16);
    assert_ok!(u16, u16::MIN, u32, u32::MIN);

    assert_ok!(u32, u32::MAX, u32, u32::MAX);
    assert_ok!(u32, u32::MIN, u32, u32::MIN);

    assert_ok!(u8, 0, u64, 0);
    assert_ok!(u8, u8::MAX, u64, (u8::MAX as u64) << 56);
    assert_ok!(u8, u8::MIN, u64, u64::MIN);

    assert_ok!(u16, 0, u64, 0);
    assert_ok!(u16, u16::MAX, u64, (u16::MAX as u64) << 48);
    assert_ok!(u16, u16::MIN, u64, u64::MIN);

    assert_ok!(u32, 0, u64, 0);
    assert_ok!(u32, u32::MAX, u64, (u32::MAX as u64) << 32);
    assert_ok!(u32, u32::MIN, u64, u64::MIN);

    assert_ok!(u64, 0, u64, 0);
    assert_ok!(u64, u64::MAX, u64, u64::MAX);
    assert_ok!(u64, u64::MIN, u64, u64::MIN);
}

#[test]
fn test_lossless_signed() {
    assert_ok!(i8, 0, i16, 0);
    assert_ok!(i8, 0x13, i16, 0x1300);
    assert_ok!(i8, -0x13, i16, -0x1300);
    assert_ok!(i8, i8::MAX, i16, (i8::MAX as i16) << 8);
    assert_ok!(i8, i8::MIN, i16, i16::MIN);

    assert_ok!(i8, 0, i32, 0);
    assert_ok!(i8, 0x13, i32, 0x13000000);
    assert_ok!(i8, -0x13, i32, -0x13000000);
    assert_ok!(i8, i8::MAX, i32, (i8::MAX as i32) << 24);
    assert_ok!(i8, i8::MIN, i32, i32::MIN);

    assert_ok!(i16, 0, i32, 0);
    assert_ok!(i16, 0x13, i32, 0x130000);
    assert_ok!(i16, 0x1337, i32, 0x13370000);
    assert_ok!(i16, -0x13, i32, -0x130000);
    assert_ok!(i16, -0x1337, i32, -0x13370000);
    assert_ok!(i16, i16::MAX, i32, (i16::MAX as i32) << 16);
    assert_ok!(i16, i16::MIN, i32, i32::MIN);

    assert_ok!(i8, 0, i64, 0);
    assert_ok!(i8, 0x13, i64, 0x1300000000000000);
    assert_ok!(i8, -0x13, i64, -0x1300000000000000);
    assert_ok!(i8, i8::MAX, i64, (i8::MAX as i64) << 56);
    assert_ok!(i8, i8::MIN, i64, i64::MIN);

    assert_ok!(i16, 0, i64, 0);
    assert_ok!(i16, 0x1337, i64, 0x1337000000000000);
    assert_ok!(i16, -0x1337, i64, -0x1337000000000000);
    assert_ok!(i16, i16::MAX, i64, (i16::MAX as i64) << 48);
    assert_ok!(i16, i16::MIN, i64, i64::MIN);

    assert_ok!(i32, 0, i64, 0);
    assert_ok!(i32, 0x1337, i64, 0x133700000000);
    assert_ok!(i32, -0x1337, i64, -0x133700000000);
    assert_ok!(i32, i32::MAX, i64, (i32::MAX as i64) << 32);
    assert_ok!(i32, i32::MIN, i64, i64::MIN);
}

macro_rules! assert_err {
    ($in_ty:ty, $in:expr => $ty:ty) => {
        let o: $in_ty = $in;
        assert_eq!(<$ty>::try_translate(o), Err(IntTranslationError));
    };
}

#[test]
fn test_failing_unsigned() {
    assert_err!(u16, 0x1301 => u8);
    assert_err!(u32, 0x13000001 => u8);
    assert_err!(u64, 0x1300000000000001 => u8);
    assert_err!(u32, 0x13000001 => u16);
    assert_err!(u64, 0x1300000000000001 => u16);
    assert_err!(u64, 0x1300000000000001 => u32);
}

#[test]
fn test_failing_signed() {
    assert_err!(i16, 0x1301 => i8);
    assert_err!(i32, 0x13000001 => i8);
    assert_err!(i64, 0x1300000000000001 => i8);
    assert_err!(i32, 0x13000001 => i16);
    assert_err!(i64, 0x1300000000000001 => i16);
    assert_err!(i64, 0x1300000000000001 => i32);
}
