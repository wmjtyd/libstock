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
/// - `u32`: 5-byte encoding & decoding.
/// - `u64`: 10-byte encoding & decoding.
///
/// # Examples
///
/// ```
/// use wmjtyd_libstock::data::hex::{NumToBytesExt, HexDataError};
///
/// assert!(matches!(u32::encode_bytes("1280"), Ok(v) if v == [0, 0, 5, 0, 0]));
/// assert_eq!(u32::decode_bytes(&[0, 0, 5, 0, 0]).to_string(), "1280");
///
/// assert!(matches!(u64::encode_bytes("12800000000"), Ok(v) if v == [0, 0, 0, 0, 2, 250, 240, 128, 0, 0]));
/// assert_eq!(u64::decode_bytes(&[0, 0, 0, 0, 2, 250, 240, 128, 0, 0]).to_string(), "12800000000");
/// ```
pub trait NumToBytesExt<const LEN: usize> {
    /// Encode a number string to bytes.
    fn encode_bytes(value: &str) -> HexDataResult<[u8; LEN]>;

    /// Decode the specified bytes to a [`Decimal`].
    fn decode_bytes(value: &[u8; LEN]) -> Decimal;
}

impl NumToBytesExt<5> for u32 {
    /// Encode a number string to bytes.
    ///
    /// # Example
    ///
    /// ```
    /// use wmjtyd_libstock::data::hex::{NumToBytesExt, HexDataError};
    ///
    /// assert!(matches!(u32::encode_bytes("1280"), Ok(v) if v == [0, 0, 5, 0, 0]));
    /// assert!(matches!(u32::encode_bytes("25600"), Ok(v) if v == [0, 0, 100, 0, 0]));
    /// assert!(matches!(u32::encode_bytes("512000"), Ok(v) if v == [0, 7, 208, 0, 0]));
    /// assert!(matches!(u32::encode_bytes("10240000"), Ok(v) if v == [0, 156, 64, 0, 0]));
    ///
    /// assert!(matches!(u32::encode_bytes("512.000"), Ok(v) if v == [0, 7, 208, 0, 3]));
    /// assert!(matches!(u32::encode_bytes("512.001"), Ok(v) if v == [0, 7, 208, 1, 3]));
    /// assert!(matches!(u32::encode_bytes("512.016"), Ok(v) if v == [0, 7, 208, 16, 3]));
    ///
    /// // Encode a number string to i8 bytes.
    /// {
    ///     let encoded_u8 = u32::encode_bytes("10000000.99").unwrap();
    ///     let encoded_i8 = unsafe { std::slice::from_raw_parts(encoded_u8.as_ptr() as *const i8, encoded_u8.len()) };
    ///     assert_eq!(encoded_i8, &[59i8, -102, -54, 99, 2]);
    /// }
    ///
    /// assert!(matches!(
    ///     u32::encode_bytes("Hello!"),
    ///     Err(HexDataError::StrNumParseError(_))
    /// ));
    /// ```
    fn encode_bytes(value: &str) -> HexDataResult<[u8; 5]> {
        let mut result = [0u8; 5];

        // if value.find("E-") != Some(0) {
        //     let split: Vec<&str> = value.split("E-").collect();
        //     let a = split[1];
        //     e = a.parse().unwrap();
        //     value = split[0].to_string();
        // }

        let (num_str, scale) = float_to_num_with_scale(value);

        let num = num_str.parse::<Self>()?.to_be_bytes();
        result[..4].copy_from_slice(&num);
        result[4] = scale;

        Ok(result)
    }

    /// Decode the specified bytes to a [`Decimal`].
    ///
    /// # Example
    ///
    /// ```
    /// use wmjtyd_libstock::data::hex::{NumToBytesExt, HexDataError};
    ///
    /// assert_eq!(u32::decode_bytes(&[0, 0, 5, 0, 0]).to_string(), "1280");
    /// assert_eq!(u32::decode_bytes(&[0, 0, 100, 0, 0]).to_string(), "25600");
    /// assert_eq!(
    ///     u32::decode_bytes(&[0, 7, 208, 0, 0]).to_string(),
    ///     "512000"
    /// );
    /// assert_eq!(
    ///     u32::decode_bytes(&[0, 156, 64, 0, 0]).to_string(),
    ///     "10240000"
    /// );
    ///
    /// assert_eq!(
    ///     u32::decode_bytes(&[0, 7, 208, 0, 3]).to_string(),
    ///     "512.000"
    /// );
    /// assert_eq!(
    ///     u32::decode_bytes(&[0, 7, 208, 1, 3]).to_string(),
    ///     "512.001"
    /// );
    /// assert_eq!(
    ///     u32::decode_bytes(&[0, 7, 208, 16, 3]).to_string(),
    ///     "512.016"
    /// );
    ///
    /// // Decode a i8 bytes to a number string
    /// {
    ///     let raw_i8 = [59i8, -102, -54, 99, 2];
    ///     let raw_u8: &[u8; 5] = unsafe { std::slice::from_raw_parts(raw_i8.as_ptr() as *const u8, raw_i8.len()) }
    ///         .try_into().expect("failed to convert [i8; 5] to [u8; 5]");
    ///     assert_eq!(u32::decode_bytes(raw_u8).to_string(), "10000000.99");
    /// }
    /// ```
    fn decode_bytes(value: &[u8; 5]) -> Decimal {
        let num_part = Self::from_be_bytes(*arrayref::array_ref![value, 0, 4]) as i64;

        let scale_part = u32::from_be_bytes({
            let raw = value[4];

            [0, 0, 0, raw]
        });

        Decimal::new(num_part, scale_part)
    }
}

impl NumToBytesExt<10> for u64 {
    // WIP: examples
    fn encode_bytes(value: &str) -> HexDataResult<[u8; 10]> {
        let mut result = [0u8; 10];

        let (num_str, scale) = float_to_num_with_scale(value);

        let num = num_str.parse::<Self>()?.to_be_bytes();
        result[1..9].copy_from_slice(&num);
        result[9] = scale;

        Ok(result)
    }

    // WIP: examples
    fn decode_bytes(value: &[u8; 10]) -> Decimal {
        let num_part = Self::from_be_bytes(*arrayref::array_ref![value, 1, 8]) as i64;

        let scale_part = u32::from_be_bytes({
            let raw = value[9];

            [0, 0, 0, raw]
        });

        Decimal::new(num_part, scale_part)
    }
}

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

#[deprecated(note = "use u32::encode_bytes instead")]
pub fn encode_num_to_bytes(value: &str) -> HexDataResult<[u8; 5]> {
    u32::encode_bytes(value)
}

#[deprecated(note = "use u32::decode_bytes instead")]
pub fn decode_bytes_to_num(value: &[u8; 5]) -> Decimal {
    u32::decode_bytes(value)
}

#[derive(thiserror::Error, Debug)]
pub enum HexDataError {
    #[error("unable to encode a string to number: {0}")]
    StrNumParseError(#[from] ParseIntError),
}

pub type HexDataResult<T> = Result<T, HexDataError>;
