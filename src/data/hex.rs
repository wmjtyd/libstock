//! Methods for operating with hexadecimal strings.

use std::num::ParseIntError;

use rust_decimal::prelude::*;

/// Convert a UNIX timestamp in `ms` to a 6-byte hex string.
///
/// Note that we will do the following check in debug mode:
///
/// - Make sure the encoded `u64` number do not use the 0 & 1 byte.
///
/// # Example
///
/// ```
/// use wmjtyd_libstock::data::hex::unix_ms_to_six_byte_hex;
///
/// assert_eq!(unix_ms_to_six_byte_hex(1656991593000), [1, 129, 204, 101, 50, 40]);
/// ```
pub fn unix_ms_to_six_byte_hex(timestamp: u64) -> [u8; 6] {
    let encoded = timestamp.to_be_bytes();

    // Make sure the encoded number do not use the 0 & 1 byte.
    debug_assert_eq!(encoded[0], 0);
    debug_assert_eq!(encoded[1], 0);

    *arrayref::array_ref![encoded, 2, 6]
}

/// Convert 6-byte hex string to the UNIX timestamp in `ms`.
///
/// Note: we convert to `u64` instead of `u128`, as the latter
/// is not native and may introduce performance degradation.
/// Besides, `u128` is meaningless as the encoded_timestamp only support
/// numbers that can be represented with 6-byte hex string, as known as
/// the subset of `u64`.
///
/// # Example
///
/// ```
/// use wmjtyd_libstock::data::hex::six_byte_hex_to_unix_ms;
///
/// assert_eq!(six_byte_hex_to_unix_ms(&[1, 129, 204, 101, 50, 40]), 1656991593000);
/// ```
pub fn six_byte_hex_to_unix_ms(encoded_timestamp: &[u8; 6]) -> u64 {
    let eight_byte_encoded = {
        let mut buf = [0u8; 8];
        arrayref::array_mut_ref![buf, 2, 6].copy_from_slice(encoded_timestamp);

        buf
    };

    u64::from_be_bytes(eight_byte_encoded)
}

/// Encode a number to bytes, or decode bytes to number.
///
/// We currently support two variants:
///
/// - `i32` or `u32`: 5-byte encoding & decoding.
/// - `i64` or `u64`: 10-byte encoding & decoding.
///
/// # Examples
///
/// ```
/// use wmjtyd_libstock::data::hex::{NumToBytesExt, HexDataError};
///
/// assert!(matches!(i32::encode_bytes("1280"), Ok(v) if v == [0, 0, 5, 0, 0]));
/// assert_eq!(i32::decode_bytes(&[0, 0, 5, 0, 0]).to_string(), "1280");
///
/// assert!(matches!(i64::encode_bytes("12800000000"), Ok(v) if v == [0, 0, 0, 0, 2, 250, 240, 128, 0, 0]));
/// assert_eq!(i64::decode_bytes(&[0, 0, 0, 0, 2, 250, 240, 128, 0, 0]).to_string(), "12800000000");
/// ```
pub trait NumToBytesExt<const LEN: usize> {
    /// Encode a number string to [`u8`] bytes.
    fn encode_bytes(value: &str) -> HexDataResult<[u8; LEN]>;

    /// Decode the specified [`u8`] bytes to a [`Decimal`].
    fn decode_bytes(value: &[u8; LEN]) -> Decimal;

    /// Encode a number string to [`i8`] bytes safely.
    fn encode_i8_bytes(value: &str) -> HexDataResult<[i8; LEN]> {
        let encoded_u8 = Self::encode_bytes(value)?;

        Ok(encoded_u8.map(|v| v as i8))
    }

    /// Decode a number string to [`i8`] bytes safely.
    fn decode_i8_bytes(value: &[i8; LEN]) -> Decimal {
        let encoded_u8 = value.map(|v| v as u8);

        Self::decode_bytes(&encoded_u8)
    }
}

// Really dirty???
trait WtfUnsignedPolyfill {
    type SignedType;
    type UnsignedType;

    fn signum(&self) -> Self::SignedType {
        unreachable!()
    }
    fn unsigned_abs(&self) -> Self::UnsignedType {
        unreachable!()
    }
}

impl WtfUnsignedPolyfill for u32 {
    type SignedType = i32;
    type UnsignedType = u32;
}
impl WtfUnsignedPolyfill for u64 {
    type SignedType = i64;
    type UnsignedType = u64;
}

macro_rules! build_opt_enc_mod {
    // Function definition for encoding.
    (@encfn $num_type:ty, $len: expr, $sign_needed: expr) => {
        fn encode_bytes(value: &str) -> HexDataResult<[u8; $len]> {
            let mut result = [0u8; $len];

            let (num_str, scale) = float_to_num_with_scale(value);

            let num = num_str.parse::<$num_type>()?;

            // Don't mind this if-block; compiler will optimize this.
            //
            // For example:
            //     if (true) { 1 } else { 0 }
            // Rust will compile it to:
            //     1
            let (bytes, scale) = if ($sign_needed) {
                let sign = num.signum();
                let bytes = num.unsigned_abs().to_be_bytes();
                (
                    bytes,
                    if sign == -1 {
                        // 0x7f is mask
                        // 0x80 is sign
                        (scale & 0x7f) | 0x80
                    } else {
                        scale
                    }
                )
            } else {
                (num.to_be_bytes(), scale)
            };

            // The begin offset of our data.
            let begin = build_opt_enc_mod!(@decbegin $len);
            // The index next to the end of number data.
            let end = begin + build_opt_enc_mod!(@decsize $len);
            // The end of our data, which is our scale.
            let scale_idx = end;

            result[begin..end].copy_from_slice(&bytes);
            result[scale_idx] = scale;

            Ok(result)
        }
    };

    // arrayref's begin offset
    (@decbegin $len:expr) => {
        if $len == 5 {
            0
        } else {
            1
        }
    };

    // arrayref's pick length
    (@decsize $len:expr) => {
        if $len == 5 {
            4
        } else {
            8
        }
    };

    // Function definition for decoding.
    (@decfn $num_type:ty, $len: expr, $sign_needed: expr) => {
        fn decode_bytes(value: &[u8; $len]) -> Decimal {
            let num_part = Self::from_be_bytes(*arrayref::array_ref![
                value,
                build_opt_enc_mod!(@decbegin $len),
                build_opt_enc_mod!(@decsize $len)
            ]) as i64;

            // The index where scale is.
            let scale_idx =
                build_opt_enc_mod!(@decbegin $len) +
                build_opt_enc_mod!(@decsize $len);

            // Don't mind this if-block; compiler will optimize this.
            //
            // For example:
            //     if (true) { 1 } else { 0 }
            // Rust will compile it to:
            //     1
            if ($sign_needed) {
                // Principle
                // =========
                //
                // 1 1 0 1 - 1 1 1 0
                // 0 1 1 1 - 1 1 1 1  mask (&)
                // 0 1 0 1 - 1 1 1 0
                // ??????  >> 7
                // ??????  0 1 1 1 - 1 1 1 1  mask (&)

                // float index num
                let raw = value[scale_idx];

                // false is Positive numbers
                // true is Negative numbers
                let sign = raw >> 7 != 0;

                // 0x7f is mask
                let scale_part = (raw & 0x7f) as u32;

                let mut decimal = Decimal::new(num_part, scale_part);
                decimal.set_sign_negative(sign);
                decimal
            } else {
                // This is the optimized code for unsigned number
                // It doesn't have some unnecessary determination.
                let scale_part = u32::from_be_bytes([0, 0, 0, value[scale_idx]]);
                let decimal = Decimal::new(num_part, scale_part);
                decimal
            }
        }
    };

    // Function definition for a impl block.
    (@impl $num_type:ty, $len: expr, $sign_needed: expr) => {
        build_opt_enc_mod!(@encfn $num_type, $len, $sign_needed);
        build_opt_enc_mod!(@decfn $num_type, $len, $sign_needed);
    };

    ($num_type:ty, $len: expr, $sign_needed: expr) => {
        impl NumToBytesExt<$len> for $num_type {
            build_opt_enc_mod!(@impl $num_type, $len, $sign_needed);
        }
    };
}

build_opt_enc_mod!(i32, 5, true);
build_opt_enc_mod!(i64, 10, true);
build_opt_enc_mod!(u32, 5, false);
build_opt_enc_mod!(u64, 10, false);

/// Extract a float number string to a number string with a scale.
type NumberString = String;
type Scale = u8;
fn float_to_num_with_scale(value: &str) -> (NumberString, Scale) {
    const E: usize = 0;

    let scale_point_indicator = match value.find('.') {
        Some(idx) => value.len() - idx - 1 + E,
        None => 0,
    } as u8;

    let value = value.replace('.', "");
    (value, scale_point_indicator)
}

#[derive(thiserror::Error, Debug)]
pub enum HexDataError {
    #[error("unable to encode a string to number: {0}")]
    StrNumParseError(#[from] ParseIntError),
}

pub type HexDataResult<T> = Result<T, HexDataError>;

#[cfg(test)]
mod tests {
    // For readability.
    #![allow(clippy::identity_op)]

    use super::{HexDataError, NumToBytesExt};

    const SIGN: u8 = 0x80;

    // Meaningless behavior
    // For readability of the code only
    const NOT_SIGN: u8 = 0x00;

    #[test]
    fn test_5b_encode() {
        assert_eq!(
            i32::encode_bytes("1280").unwrap(),
            [0, 0, 5, 0, 0 | NOT_SIGN]
        );
        assert_eq!(
            i32::encode_bytes("25600").unwrap(),
            [0, 0, 100, 0, 0 | NOT_SIGN]
        );
        assert_eq!(
            i32::encode_bytes("512000").unwrap(),
            [0, 7, 208, 0, 0 | NOT_SIGN]
        );
        assert_eq!(
            i32::encode_bytes("10240000").unwrap(),
            [0, 156, 64, 0, 0 | NOT_SIGN]
        );
        assert_eq!(
            i32::encode_bytes("-10240000").unwrap(),
            [0, 156, 64, 0, 0 | SIGN]
        );

        assert_eq!(
            i32::encode_bytes("512.000").unwrap(),
            [0, 7, 208, 0, 3 | NOT_SIGN]
        );
        assert_eq!(
            i32::encode_bytes("512.001").unwrap(),
            [0, 7, 208, 1, 3 | NOT_SIGN]
        );
        assert_eq!(
            i32::encode_bytes("512.016").unwrap(),
            [0, 7, 208, 16, 3 | NOT_SIGN]
        );
        assert_eq!(
            i32::encode_bytes("-10240000.1").unwrap(),
            [6, 26, 128, 1, 1 | SIGN]
        );
        assert_eq!(
            i32::encode_bytes("-10240000.12").unwrap(),
            [61, 9, 0, 12, 2 | SIGN]
        );

        assert!(matches!(
            i32::encode_bytes("Hello!"),
            Err(HexDataError::StrNumParseError(_))
        ));
    }

    #[test]
    fn test_5b_decode() {
        assert_eq!(
            i32::decode_bytes(&[0, 0, 5, 0, 0x00 | NOT_SIGN]).to_string(),
            "1280"
        );
        assert_eq!(
            i32::decode_bytes(&[0, 0, 100, 0, 0x00 | NOT_SIGN]).to_string(),
            "25600"
        );
        assert_eq!(
            i32::decode_bytes(&[0, 7, 208, 0, 0x00 | NOT_SIGN]).to_string(),
            "512000"
        );
        assert_eq!(
            i32::decode_bytes(&[0, 156, 64, 0, 0x00 | NOT_SIGN]).to_string(),
            "10240000"
        );
        // 0x00 is
        assert_eq!(
            i32::decode_bytes(&[0, 156, 64, 0, 0 | SIGN]).to_string(),
            "-10240000"
        );

        assert_eq!(
            i32::decode_bytes(&[0, 7, 208, 0, 3 | NOT_SIGN]).to_string(),
            "512.000"
        );
        assert_eq!(
            i32::decode_bytes(&[0, 7, 208, 1, 3 | NOT_SIGN]).to_string(),
            "512.001"
        );
        assert_eq!(
            i32::decode_bytes(&[0, 7, 208, 16, 3 | NOT_SIGN]).to_string(),
            "512.016"
        );
        assert_eq!(
            i32::decode_bytes(&[6, 26, 128, 1, 1 | SIGN]).to_string(),
            "-10240000.1"
        );
        assert_eq!(
            i32::decode_bytes(&[61, 9, 0, 12, 2 | SIGN]).to_string(),
            "-10240000.12"
        );
    }
}
