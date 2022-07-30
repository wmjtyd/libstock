use super::{Decimal, NumError};

/// The trait for encoding a specified bytes of hex string
/// – which is represented as `[TGT; LEN]` –
/// to a [`Decimal`] number.
///
/// Currently supports:
/// - `LEN = 5`: 5-bytes hex string. For 32-bit numbers.
/// - `LEN = 10`: 10-bytes hex string. For 64-bit numbers.
///
/// `TGT` can be one of `u8` or `i8`. It is `u8`
/// by default, but if you need to support
/// Java, you may want to change `TGT` to `i8`.
pub trait Decoder<const LEN: usize, TGT>
where
    Self: Sized,
{
    type Err;

    fn decode(src: &[TGT; LEN]) -> Result<Self, Self::Err>;
}

impl Decoder<5, i8> for Decimal {
    type Err = NumError;

    fn decode(src: &[i8; 5]) -> Result<Self, Self::Err> {
        let encoded_i8 = unsafe { std::mem::transmute(src) };

        <Self as Decoder<5, u8>>::decode(encoded_i8)
    }
}

impl Decoder<5, u8> for Decimal {
    type Err = NumError;

    fn decode(src: &[u8; 5]) -> Result<Self, Self::Err> {
        let num_part = u32::from_be_bytes(*arrayref::array_ref![src, 0, 4]);

        let signed_num = src[4];

        let (scale, is_negative) = split_signed_scale(signed_num);

        let mut decimal = Decimal::new(num_part as i64, scale as u32);
        decimal.set_sign_negative(is_negative);

        Ok(decimal)
    }
}

impl Decoder<10, u8> for Decimal {
    type Err = NumError;

    fn decode(src: &[u8; 10]) -> Result<Self, Self::Err> {
        let num_part = u64::from_be_bytes(*arrayref::array_ref![src, 1, 8]);

        let signed_num = src[9];

        let (scale, is_negative) = split_signed_scale(signed_num);

        let mut decimal = Decimal::new(num_part as i64, scale as u32);
        decimal.set_sign_negative(is_negative);

        Ok(decimal)
    }
}

/// Split the signed scale to a scale with a negative flag.
///
/// The return value is `(scale, is_negative)`.
///
/// # Principle
///
/// ```plain
/// 1 1 0 1 - 1 1 1 0
/// 0 1 1 1 - 1 1 1 1  mask (&)
/// 0 1 0 1 - 1 1 1 0
/// 正负  >> 7
/// 小數  0 1 1 1 - 1 1 1 1  mask (&)
/// ```
fn split_signed_scale(signed_scale: u8) -> (u8, bool) {
    // false is Positive numbers
    // true is Negative numbers
    let sign = signed_scale >> 7 != 0;

    // 0x7f is mask
    let scale_part = signed_scale & 0x7f;

    (scale_part, sign)
}

#[cfg(test)]
mod tests {
    // For readability.
    #![allow(clippy::identity_op)]

    use crate::data::num::test_utils::{get_basic_tests_set, NOT_SIGN, SIGN};

    #[test]
    fn test_scale_split() {
        use super::split_signed_scale;

        assert_eq!((0, false), split_signed_scale(0 | NOT_SIGN));
        assert_eq!((0, true), split_signed_scale(0 | SIGN));
        assert_eq!((1, false), split_signed_scale(1 | NOT_SIGN));
        assert_eq!((2, true), split_signed_scale(2 | SIGN));
        assert_eq!((3, false), split_signed_scale(3 | NOT_SIGN));
    }

    #[test]
    fn test_5b_decode() {
        use super::Decimal;
        use crate::data::num::Decoder;

        let tests_set = get_basic_tests_set::<5>();

        for (expected, raw) in tests_set.iter() {
            let raw = arrayref::array_ref![raw.as_slice(), 0, 5];
            let actual = Decimal::decode(raw).unwrap();
            assert_eq!(actual, *expected);
        }
    }

    #[test]
    fn test_10b_decode() {
        use super::Decimal;
        use crate::data::num::Decoder;

        let tests_set = get_basic_tests_set::<10>();

        for (expected, raw) in tests_set.iter() {
            let raw = arrayref::array_ref![raw.as_slice(), 0, 10];
            let actual = Decimal::decode(raw).unwrap();
            assert_eq!(actual, *expected);
        }
    }
}
