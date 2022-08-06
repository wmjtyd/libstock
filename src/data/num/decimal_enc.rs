use super::{Decimal, NumError};

/// The trait for encoding a [`Decimal`] number
/// to the specified bytes of hex string,
/// represented as `[TGT; LEN]`.
///
/// `LEN` Currently supports:
/// - `LEN = 5`: 5-bytes hex string. For 32-bit numbers.
/// - `LEN = 10`: 10-bytes hex string. For 64-bit numbers.
///
/// `TGT` can be one of `u8` or `i8`. It is `u8`
/// by default, but if you need to support
/// Java, you may want to change `TGT` to `i8`.
pub trait Encoder<const LEN: usize, TGT = u8>
where
    Self: Sized,
{
    type Err;

    /// Encoding a [`Decimal`] number to the
    /// specified length of hex string.
    fn encode(&self) -> Result<[TGT; LEN], Self::Err>;
}

macro_rules! i8_encoder {
    ($len:expr) => {
        impl Encoder<$len, i8> for Decimal {
            type Err = NumError;

            fn encode(&self) -> Result<[i8; $len], Self::Err> {
                let encoded_u8 = <Self as Encoder<$len, u8>>::encode(self)?;

                Ok(unsafe { std::mem::transmute(encoded_u8) })
            }
        }
    };
}

macro_rules! u8_encoder_body {
    ($self:expr, $abs_to:ty) => {{
        let is_negative = $self.is_sign_negative();
        let num_part = <$abs_to>::try_from($self.mantissa().unsigned_abs()).expect("overflow!");
        let scale_part = $self.scale() as u8;

        let num_bytes = num_part.to_be_bytes();
        let signed_scale = merge_to_signed_scale(scale_part, is_negative);

        (num_bytes, signed_scale)
    }};
}

i8_encoder!(5);
i8_encoder!(10);

impl Encoder<5, u8> for Decimal {
    type Err = NumError;

    fn encode(&self) -> Result<[u8; 5], Self::Err> {
        let mut result = [0u8; 5];

        let (num_bytes, signed_scale) = u8_encoder_body!(self, u32);

        result[0..4].copy_from_slice(&num_bytes);
        result[4] = signed_scale;

        Ok(result)
    }
}

impl Encoder<10, u8> for Decimal {
    type Err = NumError;

    fn encode(&self) -> Result<[u8; 10], Self::Err> {
        let mut result = [0u8; 10];

        let (num_bytes, signed_scale) = u8_encoder_body!(self, u64);

        result[1..9].copy_from_slice(&num_bytes);
        result[9] = signed_scale;

        Ok(result)
    }
}

/// Merge the [`u8`] scale with the negative to a *signed scale*.
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
fn merge_to_signed_scale(scale: u8, is_negative: bool) -> u8 {
    if is_negative {
        // 0x7f is mask
        // 0x80 is sign
        (scale & 0x7f) | 0x80
    } else {
        scale
    }
}

#[cfg(test)]
mod tests {
    // For readability.
    #![allow(clippy::identity_op)]

    use crate::data::num::test_utils::{get_basic_tests_set, NOT_SIGN, SIGN};

    #[test]
    fn test_scale_merge() {
        use super::merge_to_signed_scale;

        assert_eq!(merge_to_signed_scale(0, false), 0 | NOT_SIGN);
        assert_eq!(merge_to_signed_scale(0, true), 0 | SIGN);
        assert_eq!(merge_to_signed_scale(1, false), 1 | NOT_SIGN);
        assert_eq!(merge_to_signed_scale(2, true), 2 | SIGN);
        assert_eq!(merge_to_signed_scale(3, false), 3 | NOT_SIGN);
    }

    #[test]
    fn test_5b_encode() {
        use crate::data::num::Encoder;

        let tests_set = get_basic_tests_set::<5>();

        for (raw, expected) in tests_set.iter() {
            let actual: [u8; 5] = raw.encode().unwrap();
            assert_eq!(actual.as_slice(), expected.as_slice());
        }
    }

    #[test]
    fn test_10b_encode() {
        use crate::data::num::Encoder;

        let tests_set = get_basic_tests_set::<10>();

        for (raw, expected) in tests_set.iter() {
            let actual: [u8; 10] = raw.encode().unwrap();
            assert_eq!(actual.as_slice(), expected.as_slice());
        }
    }
}
